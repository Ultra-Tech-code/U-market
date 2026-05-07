use anchor_lang::prelude::*;
use crate::state::UserProfile;
use crate::errors::UmarketError;
use crate::events::ProfileUpdated;

pub fn handler(ctx: Context<UpdateProfile>, location: String, mail: String) -> Result<()> {
    require!(!location.is_empty(), UmarketError::EmptyString);
    require!(!mail.is_empty(), UmarketError::EmptyString);
    require!(location.len() <= UserProfile::MAX_LOCATION, UmarketError::EmptyString);
    require!(mail.len() <= UserProfile::MAX_MAIL, UmarketError::EmptyString);

    let profile = &mut ctx.accounts.profile;
    profile.location = location.clone();
    profile.mail = mail.clone();

    emit!(ProfileUpdated {
        creator: ctx.accounts.user.key(),
        location,
        mail,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateProfile<'info> {
    #[account(
        mut,
        seeds = [b"profile", user.key().as_ref()],
        bump = profile.bump,
        constraint = profile.owner == user.key() @ UmarketError::NotOwner,
    )]
    pub profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub user: Signer<'info>,
}