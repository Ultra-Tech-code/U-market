use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, MintTo, Mint};
use crate::state::{PlatformConfig, UserProfile, Product, Escrow};
use crate::errors::UmarketError;
use crate::events::PaymentApproved;

pub fn handler(ctx: Context<ApprovePaymentSpl>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;
    let total_amount = escrow.amount;
    require!(total_amount > 0, UmarketError::NoPaymentFound);

    let fee_pct = ctx.accounts.platform_config.fee_percentage as u64;
    let platform_fee = (total_amount * fee_pct) / 100;
    let seller_amount = total_amount - platform_fee;

    let product_id = ctx.accounts.product.product_id;
    let buyer_key = ctx.accounts.buyer.key();
    let escrow_seeds: &[&[&[u8]]] = &[&[
        b"escrow",
        &product_id.to_le_bytes(),
        buyer_key.as_ref(),
        &[escrow.bump],
    ]];

    // Transfer seller's share from escrow vault → seller token account
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.seller_token_account.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        },
        escrow_seeds,
    );
    token::transfer(cpi_ctx, seller_amount)?;

    // Transfer platform fee from escrow vault → fee recipient token account
    if platform_fee > 0 {
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.fee_recipient_token_account.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            escrow_seeds,
        );
        token::transfer(cpi_ctx, platform_fee)?;
    }

    ctx.accounts.product.in_progress -= 1;

    // Update seller stats
    let seller_profile = &mut ctx.accounts.seller_profile;
    seller_profile.recycled_count += 1;
    seller_profile.recycled_weight += escrow.amount_kg;
    seller_profile.total_payout += seller_amount;

    // Mint reward token to buyer
    let config = &ctx.accounts.platform_config;
    let config_seeds: &[&[&[u8]]] = &[&[b"platform_config", &[config.bump]]];
    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.usedy_mint.to_account_info(),
            to: ctx.accounts.buyer_usedy_ata.to_account_info(),
            authority: ctx.accounts.platform_config.to_account_info(),
        },
        config_seeds,
    );
    token::mint_to(mint_ctx, 1_000_000_000)?;

    emit!(PaymentApproved {
        buyer: ctx.accounts.buyer.key(),
        product_id,
        total_amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ApprovePaymentSpl<'info> {
    #[account(
        mut,
        close = buyer,
        seeds = [b"escrow", product.product_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump = escrow.bump,
        has_one = buyer @ UmarketError::NotOwner,
    )]
    pub escrow: Box<Account<'info, Escrow>>,

    /// Escrow-owned token vault
    #[account(
        mut,
        seeds = [b"escrow_vault", product.product_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump,
        token::mint = spl_payment_mint,
        token::authority = escrow,
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"product", product.product_id.to_le_bytes().as_ref()],
        bump = product.bump,
    )]
    pub product: Box<Account<'info, Product>>,

    #[account(
        mut,
        seeds = [b"profile", seller.key().as_ref()],
        bump = seller_profile.bump,
        constraint = seller_profile.owner == product.owner @ UmarketError::NotOwner,
    )]
    pub seller_profile: Box<Account<'info, UserProfile>>,

    #[account(
        seeds = [b"platform_config"],
        bump = platform_config.bump,
        has_one = fee_recipient,
        has_one = spl_payment_mint,
    )]
    pub platform_config: Box<Account<'info, PlatformConfig>>,

    pub spl_payment_mint: Box<Account<'info, Mint>>,

    /// CHECK: seller receives tokens
    #[account(address = product.owner)]
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
