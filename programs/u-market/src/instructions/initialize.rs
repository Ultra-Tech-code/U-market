use anchor_lang::prelude::*;
use crate::state::PlatformConfig;
use crate::errors::UmarketError;

pub fn handler(ctx: Context<Initialize>, fee_percentage: u8) -> Result<()> {
    require!(fee_percentage < 100, UmarketError::InvalidFee);

    let config = &mut ctx.accounts.platform_config;
    config.authority = ctx.accounts.authority.key();
    config.fee_recipient = ctx.accounts.fee_recipient.key();
    config.fee_percentage = fee_percentage;
    config.umarket_mint = ctx.accounts.umarket_mint.key();
    config.spl_payment_mint = ctx.accounts.spl_payment_mint.key();
    config.product_count = 0;
    config.request_count = 0;
    config.offer_count = 0;
    config.user_count = 0;
    config.dispute_buffer = 30 * 24 * 60 * 60; // 30 days default
    config.bump = ctx.bumps.platform_config;

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = PlatformConfig::LEN,
        seeds = [b"platform_config"],
        bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    /// CHECK: just stored as pubkey
    pub fee_recipient: UncheckedAccount<'info>,

    /// CHECK: reward token mint; validated off-chain
    pub umarket_mint: UncheckedAccount<'info>,

    /// CHECK: accepted SPL payment mint (e.g. USDC)
    pub spl_payment_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}