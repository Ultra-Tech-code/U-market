use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::{PlatformConfig, UserProfile, Offer, OfferEscrow};
use crate::errors::UmarketError;

pub fn handler(ctx: Context<ResolveOfferDisputeSpl>, to_seller: bool) -> Result<()> {
    let escrow = &ctx.accounts.offer_escrow;
    let clock = Clock::get()?;
    
    require!(
        clock.unix_timestamp >= escrow.created_at + ctx.accounts.platform_config.dispute_buffer,
        UmarketError::DisputeBufferNotPassed
    );

    let total_amount = escrow.amount;
    let offer_id = ctx.accounts.offer.offer_id;
    let buyer_key = ctx.accounts.buyer.key();
    let escrow_seeds: &[&[&[u8]]] = &[&[
        b"offer_escrow",
        &offer_id.to_le_bytes(),
        buyer_key.as_ref(),
        &[escrow.bump],
    ]];

    if to_seller {
        let fee_pct = ctx.accounts.platform_config.fee_percentage as u64;
        let platform_fee = (total_amount * fee_pct) / 100;
        let seller_amount = total_amount - platform_fee;

        // Transfer seller's share
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.offer_escrow_vault.to_account_info(),
                to: ctx.accounts.seller_token_account.to_account_info(),
                authority: ctx.accounts.offer_escrow.to_account_info(),
            },
            escrow_seeds,
        );
        token::transfer(cpi_ctx, seller_amount)?;

        // Transfer platform fee
        if platform_fee > 0 {
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.offer_escrow_vault.to_account_info(),
                    to: ctx.accounts.fee_recipient_token_account.to_account_info(),
                    authority: ctx.accounts.offer_escrow.to_account_info(),
                },
                escrow_seeds,
            );
            token::transfer(cpi_ctx, platform_fee)?;
        }

        ctx.accounts.offer.delivered = true;

        // Update seller stats
        let seller_profile = &mut ctx.accounts.seller_profile;
        seller_profile.recycled_count += 1;
        seller_profile.recycled_weight += total_amount / ctx.accounts.offer.price;
        seller_profile.total_payout += seller_amount;
    } else {
        // Refund buyer
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.offer_escrow_vault.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.offer_escrow.to_account_info(),
            },
            escrow_seeds,
        );
        token::transfer(cpi_ctx, total_amount)?;

        // Reset offer status
        ctx.accounts.offer.accepted = false;
        ctx.accounts.offer.accepted_by = None;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveOfferDisputeSpl<'info> {
    #[account(
        mut,
        seeds = [b"offer", offer.offer_id.to_le_bytes().as_ref()],
        bump = offer.bump,
    )]
    pub offer: Box<Account<'info, Offer>>,

    #[account(
        mut,
        close = buyer,
        seeds = [b"offer_escrow", offer.offer_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump = offer_escrow.bump,
    )]
    pub offer_escrow: Box<Account<'info, OfferEscrow>>,

    #[account(
        mut,
        seeds = [b"offer_escrow_vault", offer.offer_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump,
        token::mint = spl_payment_mint,
        token::authority = offer_escrow,
    )]
    pub offer_escrow_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"profile", seller.key().as_ref()],
        bump = seller_profile.bump,
        constraint = seller_profile.owner == offer.seller @ UmarketError::NotOwner,
    )]
    pub seller_profile: Box<Account<'info, UserProfile>>,

    #[account(
        seeds = [b"platform_config"],
        bump = platform_config.bump,
        has_one = authority,
        has_one = fee_recipient,
        has_one = spl_payment_mint,
    )]
    pub platform_config: Box<Account<'info, PlatformConfig>>,

    pub spl_payment_mint: Box<Account<'info, Mint>>,

    pub authority: Signer<'info>,

    /// CHECK: buyer receives refund
    #[account(mut, address = offer_escrow.buyer)]
    pub buyer: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = spl_payment_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: seller receives tokens
    #[account(address = offer.seller)]
    pub seller: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = spl_payment_mint,
        token::authority = seller,
    )]
    pub seller_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: platform fee recipient
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = spl_payment_mint,
    )]
    pub fee_recipient_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
