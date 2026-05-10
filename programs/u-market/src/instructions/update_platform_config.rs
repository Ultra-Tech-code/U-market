use anchor_lang::prelude::*;
use crate::state::PlatformConfig;
use crate::errors::UmarketError;

pub fn handler(
    ctx: Context<UpdatePlatformConfig>,
    fee_percentage: u8,
    new_fee_recipient: Option<Pubkey>,
    new_dispute_buffer: Option<i64>,
) -> Result<()> {
    require!(fee_percentage < 100, UmarketError::InvalidFee);

    let config = &mut ctx.accounts.platform_config;
    config.fee_percentage = fee_percentage;
    if let Some(recipient) = new_fee_recipient {
        config.fee_recipient = recipient;
    }
    if let Some(buffer) = new_dispute_buffer {
        config.dispute_buffer = buffer;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct UpdatePlatformConfig<'info> {
    #[account(
        mut,
        seeds = [b"platform_config"],
        bump = platform_config.bump,
        has_one = authority,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub authority: Signer<'info>,
}