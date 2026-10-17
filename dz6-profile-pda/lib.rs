use anchor_lang::prelude::*;

declare_id!("4Vyg54rpLnq1ybfXnUdyZotWDV7Po1bT2UU3pAuvu77c");

const MAX_USERNAME: usize = 32;
const MAX_BIO: usize = 200;

#[program]
pub mod profile_pda {
    use super::*;

    /// Создаёт профиль пользователя в PDA-аккаунте
    pub fn create_profile(ctx: Context<CreateProfile>, username: String, bio: String) -> Result<()> {
        require!(
            !username.is_empty() && username.len() <= MAX_USERNAME,
            ProfileError::InvalidUsername
        );
        require!(bio.len() <= MAX_BIO, ProfileError::BioTooLong);

        let now = Clock::get()?.unix_timestamp;
        let profile = &mut ctx.accounts.profile;
        profile.authority = ctx.accounts.user.key();
        profile.username = username;
        profile.bio = bio;
        profile.created_at = now;
        profile.updated_at = now;
        profile.bump = ctx.bumps.profile;
        msg!("Профиль создан: {}", profile.username);
        Ok(())
    }

    /// Обновляет профиль (только владелец)
    pub fn update_profile(ctx: Context<UpdateProfile>, username: String, bio: String) -> Result<()> {
        require!(
            !username.is_empty() && username.len() <= MAX_USERNAME,
            ProfileError::InvalidUsername
        );
        require!(bio.len() <= MAX_BIO, ProfileError::BioTooLong);

        let profile = &mut ctx.accounts.profile;
        profile.username = username;
        profile.bio = bio;
        profile.updated_at = Clock::get()?.unix_timestamp;
        msg!("Профиль обновлён: {}", profile.username);
        Ok(())
    }

    /// Удаляет профиль и возвращает rent владельцу
    pub fn delete_profile(_ctx: Context<DeleteProfile>) -> Result<()> {
        msg!("Профиль удалён, rent возвращён владельцу");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateProfile<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + Profile::INIT_SPACE,
        seeds = [b"profile", user.key().as_ref()],
        bump
    )]
    pub profile: Account<'info, Profile>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    #[account(
        mut,
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority @ ProfileError::Unauthorized
    )]
    pub profile: Account<'info, Profile>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeleteProfile<'info> {
    #[account(
        mut,
        seeds = [b"profile", authority.key().as_ref()],
        bump = profile.bump,
        has_one = authority @ ProfileError::Unauthorized,
        close = authority
    )]
    pub profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Profile {
    pub authority: Pubkey, // 32 байта — владелец профиля
    #[max_len(32)]
    pub username: String, // 4 + 32 байта
    #[max_len(200)]
    pub bio: String, // 4 + 200 байт
    pub created_at: i64, // 8 байт
    pub updated_at: i64, // 8 байт
    pub bump: u8,        // 1 байт
}

#[error_code]
pub enum ProfileError {
    #[msg("Имя должно быть от 1 до 32 байт")]
    InvalidUsername,
    #[msg("Био не может быть длиннее 200 байт")]
    BioTooLong,
    #[msg("Только владелец может изменять профиль")]
    Unauthorized,
}
