use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, UserProfile, UserType};
use crate::errors::UmarketError;
use crate::events::ProfileCreated;

pub fn handler(
    ctx: Context<CreateProfile>,
    name: String,
    location: String,
    mail: String,
    user_type: UserType,
) -> Result<()> {
    require!(!name.is_empty(), UmarketError::EmptyString);
    require!(!location.is_empty(), UmarketError::EmptyString);
    require!(!mail.is_empty(), UmarketError::EmptyString);
    require!(name.len() <= UserProfile::MAX_NAME, UmarketError::EmptyString);
    require!(location.len() <= UserProfile::MAX_LOCATION, UmarketError::EmptyString);
    require!(mail.len() <= UserProfile::MAX_MAIL, UmarketError::EmptyString);

    let config = &mut ctx.accounts.platform_config;
    config.user_count = config.user_count.checked_add(1).ok_or(UmarketError::Overflow)?;
    let profile_id = config.user_count;

    let profile = &mut ctx.accounts.profile;
    profile.owner = ctx.accounts.user.key();
    profile.profile_id = profile_id;
    profile.name = name.clone();
    profile.location = location;
    profile.mail = mail;
    profile.user_type = user_type.clone();
    profile.recycled_count = 0;
    profile.recycled_weight = 0;
    profile.total_payout = 0;
    profile.bump = ctx.bumps.profile;

    emit!(ProfileCreated {
        creator: ctx.accounts.user.key(),
        name,
        profile_id,
        user_type,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CreateProfile<'info> {
    #[account(
        init,
        payer = user,
        space = UserProfile::LEN,
        seeds = [b"profile", user.key().as_ref()],
        bump,
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(
        mut,
        seeds = [b"platform_config"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}