use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::{PlatformConfig, UserProfile, Product, Escrow};
use crate::errors::UmarketError;

pub fn handler(ctx: Context<ResolveProductDisputeSpl>, to_seller: bool) -> Result<()> {
    let escrow = &ctx.accounts.escrow;
    let clock = Clock::get()?;
    
    require!(
        clock.unix_timestamp >= escrow.created_at + ctx.accounts.platform_config.dispute_buffer,
        UmarketError::DisputeBufferNotPassed
    );

    let total_amount = escrow.amount;
    let product_id = ctx.accounts.product.product_id;
    let buyer_key = ctx.accounts.buyer.key();
    let escrow_seeds: &[&[&[u8]]] = &[&[
        b"escrow",
        &product_id.to_le_bytes(),
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
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.seller_token_account.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            escrow_seeds,
        );
        token::transfer(cpi_ctx, seller_amount)?;

        // Transfer platform fee
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
    } else {
        // Refund buyer
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            escrow_seeds,
        );
        token::transfer(cpi_ctx, total_amount)?;

        // Return product weight
        ctx.accounts.product.total_weight += escrow.amount_kg;
        ctx.accounts.product.sold -= escrow.amount_kg;
        ctx.accounts.product.in_progress -= 1;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveProductDisputeSpl<'info> {
    #[account(
        mut,
        close = buyer,
        seeds = [b"escrow", product.product_id.to_le_bytes().as_ref(), buyer.key().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Box<Account<'info, Escrow>>,

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
        has_one = authority,
        has_one = fee_recipient,
        has_one = spl_payment_mint,
    )]
    pub platform_config: Box<Account<'info, PlatformConfig>>,

    pub spl_payment_mint: Box<Account<'info, Mint>>,

    pub authority: Signer<'info>,

    /// CHECK: buyer receives refund
    #[account(mut, address = escrow.buyer)]
    pub buyer: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = spl_payment_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,

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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
