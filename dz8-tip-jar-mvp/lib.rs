use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("H3qZWkqCEYDRyuTpHYLk8sChPFAqjWsnSTq5eUG4uGAQ");

const MAX_TITLE: usize = 40;
const MAX_MESSAGE: usize = 64;

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
}
