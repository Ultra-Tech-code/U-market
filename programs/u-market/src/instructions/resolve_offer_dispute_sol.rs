use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, UserProfile, Offer, OfferEscrow};
use crate::errors::UmarketError;

pub fn handler(ctx: Context<ResolveOfferDisputeSol>, to_seller: bool) -> Result<()> {
    let escrow = &ctx.accounts.offer_escrow;
    let clock = Clock::get()?;
    
    require!(
        clock.unix_timestamp >= escrow.created_at + ctx.accounts.platform_config.dispute_buffer,
        UmarketError::DisputeBufferNotPassed
    );

    let total_amount = escrow.amount;

    if to_seller {
        let fee_pct = ctx.accounts.platform_config.fee_percentage as u64;
        let platform_fee = (total_amount * fee_pct) / 100;
        let seller_amount = total_amount - platform_fee;

        // Pay seller
        **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= seller_amount;
        **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += seller_amount;

        // Pay platform fee
        if platform_fee > 0 {
            **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= platform_fee;
            **ctx.accounts.fee_recipient.to_account_info().try_borrow_mut_lamports()? += platform_fee;
        }

        ctx.accounts.offer.delivered = true;

        // Update seller stats
        let seller_profile = &mut ctx.accounts.seller_profile;
        seller_profile.recycled_count += 1;
        // price is per unit, so amount / price is quantity
        seller_profile.recycled_weight += total_amount / ctx.accounts.offer.price;
        seller_profile.total_payout += seller_amount;
    } else {
        // Refund buyer
        **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= total_amount;
        **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()? += total_amount;
        
        // Reset offer status?
        ctx.accounts.offer.accepted = false;
        ctx.accounts.offer.accepted_by = None;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveOfferDisputeSol<'info> {
    #[account(
        mut,
        seeds = [b"offer", offer.offer_id.to_le_bytes().as_ref()],
        bump = offer.bump,
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        close = buyer,
        seeds = [b"offer_escrow", offer.offer_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump = offer_escrow.bump,
    )]
    pub offer_escrow: Account<'info, OfferEscrow>,

    #[account(
        mut,
        seeds = [b"profile", seller.key().as_ref()],
        bump = seller_profile.bump,
        constraint = seller_profile.owner == offer.seller @ UmarketError::NotOwner,
    )]
    pub seller_profile: Account<'info, UserProfile>,

    #[account(
        seeds = [b"platform_config"],
        bump = platform_config.bump,
        has_one = authority,
        has_one = fee_recipient,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub authority: Signer<'info>,

    /// CHECK: seller receives lamports
    #[account(mut, address = offer.seller)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: buyer receives refund
    #[account(mut, address = offer_escrow.buyer)]
    pub buyer: UncheckedAccount<'info>,

    /// CHECK: platform fee recipient
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
