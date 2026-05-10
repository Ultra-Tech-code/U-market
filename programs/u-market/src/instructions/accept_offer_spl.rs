use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::{PlatformConfig, UserProfile, ProductRequest, Offer, OfferEscrow, PaymentMode};
use crate::errors::UmarketError;
use crate::events::OfferAccepted;

pub fn handler(ctx: Context<AcceptOfferSpl>) -> Result<()> {
    let offer = &mut ctx.accounts.offer;
    require!(!offer.accepted, UmarketError::OfferAlreadyAccepted);

    let clock = Clock::get()?;
    require!(offer.deadline > clock.unix_timestamp, UmarketError::OfferExpired);

    let request = &ctx.accounts.request;
    require!(
        matches!(request.payment_mode, PaymentMode::Spl),
        UmarketError::PaymentModeMismatch
    );

    let total_amount = offer.price * offer.quantity;

    // Transfer SPL from buyer → offer escrow vault
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.offer_escrow_vault.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, total_amount)?;

    offer.accepted = true;
    offer.accepted_by = Some(ctx.accounts.buyer.key());

    let escrow = &mut ctx.accounts.offer_escrow;
    escrow.buyer = ctx.accounts.buyer.key();
    escrow.offer_id = offer.offer_id;
    escrow.amount = total_amount;
    escrow.payment_mode = PaymentMode::Spl;
    escrow.created_at = Clock::get()?.unix_timestamp;
    escrow.bump = ctx.bumps.offer_escrow;

    emit!(OfferAccepted {
        buyer: ctx.accounts.buyer.key(),
        offer_id: offer.offer_id,
        amount: total_amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AcceptOfferSpl<'info> {
    #[account(mut, seeds = [b"offer", offer.offer_id.to_le_bytes().as_ref()], bump = offer.bump)]
    pub offer: Account<'info, Offer>,

    #[account(seeds = [b"request", request.request_id.to_le_bytes().as_ref()], bump = request.bump, constraint = request.request_id == offer.request_id @ UmarketError::InvalidId)]
    pub request: Account<'info, ProductRequest>,

    #[account(init, payer = buyer, space = OfferEscrow::LEN, seeds = [b"offer_escrow", offer.offer_id.to_le_bytes().as_ref(), buyer.key().as_ref()], bump)]
    pub offer_escrow: Account<'info, OfferEscrow>,

    #[account(init, payer = buyer, token::mint = spl_payment_mint, token::authority = offer_escrow, seeds = [b"offer_escrow_vault", offer.offer_id.to_le_bytes().as_ref(), buyer.key().as_ref()], bump)]
    pub offer_escrow_vault: Account<'info, TokenAccount>,

    #[account(mut, token::mint = spl_payment_mint, token::authority = buyer)]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(seeds = [b"profile", buyer.key().as_ref()], bump = buyer_profile.bump, constraint = buyer_profile.can_buy() @ UmarketError::UnauthorizedRole)]
    pub buyer_profile: Account<'info, UserProfile>,

    #[account(seeds = [b"platform_config"], bump = platform_config.bump, has_one = spl_payment_mint)]
    pub platform_config: Account<'info, PlatformConfig>,

    pub spl_payment_mint: Account<'info, Mint>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
