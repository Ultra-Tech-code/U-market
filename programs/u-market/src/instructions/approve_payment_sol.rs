use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, MintTo};
use crate::state::{PlatformConfig, UserProfile, Product, Escrow};
use crate::errors::UmarketError;
use crate::events::PaymentApproved;

pub fn handler(ctx: Context<ApprovePaymentSol>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;
    let total_amount = escrow.amount;
    require!(total_amount > 0, UmarketError::NoPaymentFound);

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
    let profile_id = ctx.accounts.seller_profile.profile_id;
    let seller_profile = &mut ctx.accounts.seller_profile;
    seller_profile.recycled_count += 1;
    seller_profile.recycled_weight += escrow.amount_kg;
    seller_profile.total_payout += seller_amount;

    // Mint reward token to buyer
    let config = &ctx.accounts.platform_config;
    let seeds: &[&[&[u8]]] = &[&[b"platform_config", &[config.bump]]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.usedy_mint.to_account_info(),
            to: ctx.accounts.buyer_usedy_ata.to_account_info(),
            authority: ctx.accounts.platform_config.to_account_info(),
        },
        seeds,
    );
    token::mint_to(cpi_ctx, 1_000_000_000)?;

    let product_id = ctx.accounts.product.product_id;
    emit!(PaymentApproved {
        buyer: ctx.accounts.buyer.key(),
        product_id,
        total_amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ApprovePaymentSol<'info> {
    #[account(
        mut,
        close = buyer,   // refund rent to buyer after closing
        seeds = [b"escrow", product.product_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump = escrow.bump,
        has_one = buyer @ UmarketError::NotOwner,
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
        has_one = fee_recipient,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    /// CHECK: seller receives lamports
    #[account(mut, address = product.owner)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: platform fee recipient
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,

    /// CHECK: USEDY mint
    #[account(mut)]
    pub usedy_mint: UncheckedAccount<'info>,

    /// CHECK: buyer's USEDY ATA
    #[account(mut)]
    pub buyer_usedy_ata: UncheckedAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}