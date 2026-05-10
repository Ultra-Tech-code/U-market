use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, UserProfile, Product, Escrow};
use crate::errors::UmarketError;

pub fn handler(ctx: Context<ResolveProductDisputeSol>, to_seller: bool) -> Result<()> {
    let escrow = &ctx.accounts.escrow;
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
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= seller_amount;
        **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += seller_amount;

        // Pay platform fee
        if platform_fee > 0 {
            **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= platform_fee;
            **ctx.accounts.fee_recipient.to_account_info().try_borrow_mut_lamports()? += platform_fee;
        }
        
        ctx.accounts.product.in_progress -= 1;
        
        // Update seller stats
        let seller_profile = &mut ctx.accounts.seller_profile;
        seller_profile.recycled_count += 1;
        seller_profile.recycled_weight += escrow.amount_kg;
        seller_profile.total_payout += seller_amount;
    } else {
        // Refund buyer
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= total_amount;
        **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()? += total_amount;
        
        // Return product weight
        ctx.accounts.product.total_weight += escrow.amount_kg;
        ctx.accounts.product.sold -= escrow.amount_kg;
        ctx.accounts.product.in_progress -= 1;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveProductDisputeSol<'info> {
    #[account(
        mut,
        close = buyer,
        seeds = [b"escrow", product.product_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        seeds = [b"product", product.product_id.to_le_bytes().as_ref()],
        bump = product.bump,
    )]
    pub product: Account<'info, Product>,

    #[account(
        mut,
        seeds = [b"profile", seller.key().as_ref()],
        bump = seller_profile.bump,
        constraint = seller_profile.owner == product.owner @ UmarketError::NotOwner,
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

    /// CHECK: seller receives lamports if to_seller is true
    #[account(mut, address = product.owner)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: buyer receives refund if to_seller is false
    #[account(mut, address = escrow.buyer)]
    pub buyer: UncheckedAccount<'info>,

    /// CHECK: platform fee recipient
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
