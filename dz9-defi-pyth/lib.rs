use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("H3qZWkqCEYDRyuTpHYLk8sChPFAqjWsnSTq5eUG4uGAQ");

const MAX_TITLE: usize = 40;
const MAX_MESSAGE: usize = 64;

/// Pyth Solana Receiver (владелец аккаунтов PriceUpdateV2)
const PYTH_RECEIVER: Pubkey = Pubkey::new_from_array([12, 183, 250, 187, 82, 247, 166, 72, 187, 91, 49, 125, 154, 1, 139, 144, 87, 203, 2, 71, 116, 250, 254, 1, 230, 196, 223, 152, 204, 56, 88, 129]);
/// Pyth feed id SOL/USD
const SOL_USD_FEED: [u8; 32] = [239, 13, 139, 111, 218, 44, 235, 164, 29, 161, 93, 64, 149, 209, 218, 57, 42, 13, 47, 142, 208, 198, 199, 188, 15, 76, 250, 200, 194, 128, 181, 109];
/// Цена считается устаревшей через 5 минут
const MAX_PRICE_AGE: i64 = 300;

#[program]
pub mod tip_jar {
    use super::*;

    /// Создаёт копилку (PDA) для автора
    pub fn create_jar(ctx: Context<CreateJar>, title: String, goal: u64) -> Result<()> {
        require!(!title.is_empty() && title.len() <= MAX_TITLE, JarError::InvalidTitle);
        require!(goal > 0, JarError::ZeroAmount);
        let jar = &mut ctx.accounts.jar;
        jar.owner = ctx.accounts.owner.key();
        jar.title = title;
        jar.goal = goal;
        jar.total_donated = 0;
        jar.donations = 0;
        jar.last_donor = Pubkey::default();
        jar.last_message = String::new();
        jar.bump = ctx.bumps.jar;
        msg!("Jar создан: {}", jar.title);
        Ok(())
    }

    /// Любой пользователь отправляет SOL в копилку с сообщением
    pub fn donate(ctx: Context<Donate>, amount: u64, message: String) -> Result<()> {
        require!(amount > 0, JarError::ZeroAmount);
        require!(message.len() <= MAX_MESSAGE, JarError::MessageTooLong);

        // CPI в System Program: donor -> jar PDA
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.donor.to_account_info(),
                    to: ctx.accounts.jar.to_account_info(),
                },
            ),
            amount,
        )?;

        let jar = &mut ctx.accounts.jar;
        jar.total_donated = jar.total_donated.checked_add(amount).ok_or(JarError::Overflow)?;
        jar.donations = jar.donations.checked_add(1).ok_or(JarError::Overflow)?;
        jar.last_donor = ctx.accounts.donor.key();
        jar.last_message = message.clone();

        emit!(DonationEvent {
            jar: jar.key(),
            donor: ctx.accounts.donor.key(),
            amount,
            message,
        });
        msg!("Donate: {} lamports, всего {}", amount, jar.total_donated);
        Ok(())
    }

    /// DeFi: донат в долларах. Цена SOL/USD читается из оракула Pyth прямо в программе
    pub fn donate_usd(ctx: Context<DonateUsd>, usd_cents: u64, message: String) -> Result<()> {
        require!(usd_cents > 0, JarError::ZeroAmount);
        require!(message.len() <= MAX_MESSAGE, JarError::MessageTooLong);

        // Разбор аккаунта PriceUpdateV2: discriminator(8) + write_authority(32) + verification_level + PriceFeedMessage
        let (price, conf, expo, publish_time) = {
            let data = ctx.accounts.price_update.try_borrow_data()?;
            require!(data.len() >= 8 + 32 + 2 + 32 + 28, JarError::InvalidPrice);
            let mut o = 8 + 32;
            let level = data[o];
            require!(level == 1, JarError::InvalidPrice); // 1 = Full verification
            o += 1;
            require!(data[o..o + 32] == SOL_USD_FEED, JarError::InvalidPrice);
            o += 32;
            let rd8 = |p: usize| -> [u8; 8] { data[p..p + 8].try_into().unwrap() };
            let price = i64::from_le_bytes(rd8(o));
            let conf = u64::from_le_bytes(rd8(o + 8));
            let expo = i32::from_le_bytes(data[o + 16..o + 20].try_into().unwrap());
            let publish_time = i64::from_le_bytes(rd8(o + 20));
            (price, conf, expo, publish_time)
        };
        require!(price > 0 && expo <= 0, JarError::InvalidPrice);
        let now = Clock::get()?.unix_timestamp;
        require!(now - publish_time <= MAX_PRICE_AGE, JarError::PriceTooOld);

        // lamports = cents * 1e9 * 10^(-expo) / (price * 100)
        let amount = (usd_cents as u128)
            .checked_mul(1_000_000_000u128 * 10u128.pow((-expo) as u32))
            .ok_or(JarError::Overflow)?
            / ((price as u128) * 100);
        let amount = u64::try_from(amount).map_err(|_| JarError::Overflow)?;

        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.donor.to_account_info(),
                    to: ctx.accounts.jar.to_account_info(),
                },
            ),
            amount,
        )?;

        let jar = &mut ctx.accounts.jar;
        jar.total_donated = jar.total_donated.checked_add(amount).ok_or(JarError::Overflow)?;
        jar.donations = jar.donations.checked_add(1).ok_or(JarError::Overflow)?;
        jar.last_donor = ctx.accounts.donor.key();
        jar.last_message = message.clone();

        emit!(DonationEvent {
            jar: jar.key(),
            donor: ctx.accounts.donor.key(),
            amount,
            message,
        });
        msg!(
            "Donate USD: {} cents = {} lamports | Pyth SOL/USD = {} ± {} x 10^{} (age {} s)",
            usd_cents, amount, price, conf, expo, now - publish_time
        );
        Ok(())
    }

    /// Владелец выводит SOL (rent-минимум остаётся в аккаунте)
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, JarError::ZeroAmount);
        let jar_info = ctx.accounts.jar.to_account_info();
        let rent_min = Rent::get()?.minimum_balance(jar_info.data_len());
        let available = jar_info.lamports().checked_sub(rent_min).ok_or(JarError::Overflow)?;
        require!(amount <= available, JarError::InsufficientFunds);

        **jar_info.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += amount;
        msg!("Withdraw: {} lamports", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateJar<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Jar::INIT_SPACE,
        seeds = [b"jar", owner.key().as_ref()],
        bump
    )]
    pub jar: Account<'info, Jar>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut, seeds = [b"jar", jar.owner.as_ref()], bump = jar.bump)]
    pub jar: Account<'info, Jar>,
    #[account(mut)]
    pub donor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DonateUsd<'info> {
    #[account(mut, seeds = [b"jar", jar.owner.as_ref()], bump = jar.bump)]
    pub jar: Account<'info, Jar>,
    #[account(mut)]
    pub donor: Signer<'info>,
    /// CHECK: аккаунт цены Pyth; владелец проверяется, данные разбираются вручную
    #[account(owner = PYTH_RECEIVER @ JarError::InvalidPrice)]
    pub price_update: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"jar", owner.key().as_ref()],
        bump = jar.bump,
        has_one = owner @ JarError::Unauthorized
    )]
    pub jar: Account<'info, Jar>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Jar {
    pub owner: Pubkey,
    #[max_len(40)]
    pub title: String,
    pub goal: u64,
    pub total_donated: u64,
    pub donations: u64,
    pub last_donor: Pubkey,
    #[max_len(64)]
    pub last_message: String,
    pub bump: u8,
}

#[event]
pub struct DonationEvent {
    pub jar: Pubkey,
    pub donor: Pubkey,
    pub amount: u64,
    pub message: String,
}

#[error_code]
pub enum JarError {
    #[msg("Название: от 1 до 40 байт")]
    InvalidTitle,
    #[msg("Сообщение не длиннее 64 байт")]
    MessageTooLong,
    #[msg("Сумма должна быть больше нуля")]
    ZeroAmount,
    #[msg("Переполнение")]
    Overflow,
    #[msg("Недостаточно средств в копилке")]
    InsufficientFunds,
    #[msg("Только владелец может выводить средства")]
    Unauthorized,
    #[msg("Неверный аккаунт цены Pyth")]
    InvalidPrice,
    #[msg("Цена Pyth устарела")]
    PriceTooOld,
}
