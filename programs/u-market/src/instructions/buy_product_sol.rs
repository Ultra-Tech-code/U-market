use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::{PlatformConfig, UserProfile, Product, Escrow, PaymentMode};
use crate::errors::UmarketError;
use crate::events::ProductBought;

pub fn handler(ctx: Context<BuyProductSol>, amount_kg: u64) -> Result<()> {
    require!(amount_kg > 0, UmarketError::InvalidAmount);

    let product = &mut ctx.accounts.product;
    require!(
        matches!(product.payment_mode, PaymentMode::Sol),
        UmarketError::PaymentModeMismatch
    );
    require!(product.total_weight >= amount_kg, UmarketError::NotEnoughProduct);

    let total_amount = product.discounted_price(amount_kg);

    // Transfer SOL from buyer → escrow PDA (the escrow account itself holds lamports)
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.buyer.to_account_info(),
            to: ctx.accounts.escrow.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, total_amount)?;

    product.total_weight -= amount_kg;
    product.sold += amount_kg;
    product.in_progress += 1;

    let escrow = &mut ctx.accounts.escrow;
    escrow.buyer = ctx.accounts.buyer.key();
    escrow.product_id = product.product_id;
    escrow.amount = total_amount;
    escrow.amount_kg = amount_kg;
    escrow.payment_mode = PaymentMode::Sol;
    escrow.created_at = Clock::get()?.unix_timestamp;
    escrow.bump = ctx.bumps.escrow;

    emit!(ProductBought {
        buyer: ctx.accounts.buyer.key(),
        product_id: product.product_id,
        amount_kg,
        total_paid: total_amount,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount_kg: u64)]
pub struct BuyProductSol<'info> {
    #[account(
        mut,
        seeds = [b"product", product.product_id.to_le_bytes().as_ref()],
        bump = product.bump,
    )]
    pub product: Account<'info, Product>,

    #[account(
        init,
        payer = buyer,
        space = Escrow::LEN,
        seeds = [b"escrow", product.product_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        seeds = [b"profile", buyer.key().as_ref()],
        bump = buyer_profile.bump,
        constraint = buyer_profile.can_buy() @ UmarketError::UnauthorizedRole,
    )]
    pub buyer_profile: Account<'info, UserProfile>,

    #[account(
        seeds = [b"platform_config"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}