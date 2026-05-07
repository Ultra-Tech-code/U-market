use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, MintTo};
use crate::state::{PlatformConfig, UserProfile, Offer, OfferEscrow, PaymentMode};
use crate::errors::UmarketError;
use crate::events::OfferDelivered;

pub fn handler(ctx: Context<ConfirmDelivery>) -> Result<()> {
    let offer = &mut ctx.accounts.offer;
    require!(offer.accepted, UmarketError::OfferNotAccepted);
    require!(!offer.delivered, UmarketError::OfferAlreadyDelivered);
    require!(
        offer.accepted_by == Some(ctx.accounts.buyer.key()),
        UmarketError::NotYourOffer
    );

    let escrow = &ctx.accounts.offer_escrow;
    let total_amount = escrow.amount;
    require!(total_amount > 0, UmarketError::NoPaymentFound);

    let fee_pct = ctx.accounts.platform_config.fee_percentage as u64;
    let platform_fee = (total_amount * fee_pct) / 100;
    let seller_amount = total_amount - platform_fee;

    // Pay seller (SOL from escrow lamports)
    **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= seller_amount;
    **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += seller_amount;

    // Pay platform fee
    if platform_fee > 0 {
        **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= platform_fee;
        **ctx.accounts.fee_recipient.to_account_info().try_borrow_mut_lamports()? += platform_fee;
    }

    offer.delivered = true;

    // Update seller stats
    let seller_profile = &mut ctx.accounts.seller_profile;
    seller_profile.recycled_count += 1;
    seller_profile.recycled_weight += escrow.amount / offer.price; // approximate kg
    seller_profile.total_payout += seller_amount;

    // Mint reward to buyer
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

    emit!(OfferDelivered {
        seller: ctx.accounts.seller.key(),
        offer_id: offer.offer_id,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ConfirmDelivery<'info> {
    #[account(mut, seeds = [b"offer", offer.offer_id.to_le_bytes().as_ref()], bump = offer.bump)]
    pub offer: Account<'info, Offer>,

    #[account(mut, close = buyer, seeds = [b"offer_escrow", offer.offer_id.to_le_bytes().as_ref(), buyer.key().as_ref()], bump = offer_escrow.bump, has_one = buyer @ UmarketError::NotOwner)]
    pub offer_escrow: Account<'info, OfferEscrow>,

    #[account(mut, seeds = [b"profile", seller.key().as_ref()], bump = seller_profile.bump, constraint = seller_profile.owner == offer.seller @ UmarketError::NotOwner)]
    pub seller_profile: Account<'info, UserProfile>,

    #[account(seeds = [b"platform_config"], bump = platform_config.bump, has_one = fee_recipient)]
    pub platform_config: Account<'info, PlatformConfig>,

    /// CHECK: seller receives lamports
    #[account(mut, address = offer.seller)]
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
