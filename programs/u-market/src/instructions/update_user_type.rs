use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, UserProfile, UserType};
use crate::errors::UmarketError;
use crate::events::UserTypeUpdated;

pub fn handler(ctx: Context<UpdateUserType>, new_type: UserType) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    profile.user_type = new_type.clone();

    emit!(UserTypeUpdated {
        user: profile.owner,
        new_type,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateUserType<'info> {
    #[account(
        mut,
        seeds = [b"profile", profile.owner.as_ref()],
        bump = profile.bump,
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(
        seeds = [b"platform_config"],
        bump = platform_config.bump,
        has_one = authority,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub authority: Signer<'info>,
}
