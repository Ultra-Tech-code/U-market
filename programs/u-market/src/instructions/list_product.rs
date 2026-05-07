use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, MintTo};
use crate::state::{PlatformConfig, UserProfile, Product, NegotiationTier, PaymentMode};
use crate::errors::UmarketError;
use crate::events::ProductListed;

pub fn handler(
    ctx: Context<ListProduct>,
    name: String,
    image: String,
    description: String,
    price: u64,
    weight: u64,
    payment_mode: PaymentMode,
    negotiation_tiers: Vec<NegotiationTier>,
) -> Result<()> {
    require!(!name.is_empty(), UmarketError::EmptyString);
    require!(!image.is_empty(), UmarketError::EmptyString);
    require!(!description.is_empty(), UmarketError::EmptyString);
    require!(price > 0 && weight > 0, UmarketError::InvalidPrice);
    require!(negotiation_tiers.len() <= Product::MAX_TIERS, UmarketError::TooManyTiers);

    // Validate tiers
    for tier in &negotiation_tiers {
        if tier.quantity > 0 {
            require!(
                tier.discount_percentage > 0 && tier.discount_percentage < 100,
                UmarketError::InvalidDiscount
            );
        }
    }

    let config = &mut ctx.accounts.platform_config;
    config.product_count = config.product_count.checked_add(1).ok_or(UmarketError::Overflow)?;
    let product_id = config.product_count;

    let profile = &ctx.accounts.seller_profile;

    let product = &mut ctx.accounts.product;
    product.owner = ctx.accounts.seller.key();
    product.product_id = product_id;
    product.name = name.clone();
    product.image = image;
    product.location = profile.location.clone();
    product.description = description;
    product.price = price;
    product.payment_mode = payment_mode;
    product.total_weight = weight;
    product.sold = 0;
    product.in_progress = 0;
    product.negotiation_tiers = negotiation_tiers;
    product.bump = ctx.bumps.product;

    // Mint 1 USEDY reward token for listing
    let seeds: &[&[&[u8]]] = &[&[b"platform_config", &[config.bump]]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.usedy_mint.to_account_info(),
            to: ctx.accounts.seller_usedy_ata.to_account_info(),
            authority: ctx.accounts.platform_config.to_account_info(),
        },
        seeds,
    );
    token::mint_to(cpi_ctx, 1_000_000_000)?; // 1 token with 9 decimals

    emit!(ProductListed {
        lister: ctx.accounts.seller.key(),
        product_id,
        name,
        weight,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ListProduct<'info> {
    #[account(
        init,
        payer = seller,
        space = Product::LEN,
        seeds = [b"product", platform_config.product_count.checked_add(1).unwrap().to_le_bytes().as_ref()],
        bump,
    )]
    pub product: Account<'info, Product>,

    #[account(
        mut,
        seeds = [b"platform_config"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(
        seeds = [b"profile", seller.key().as_ref()],
        bump = seller_profile.bump,
        constraint = seller_profile.can_sell() @ UmarketError::UnauthorizedRole,
    )]
    pub seller_profile: Account<'info, UserProfile>,

    /// CHECK: USEDY reward mint - authority is platform_config PDA
    #[account(mut)]
    pub usedy_mint: UncheckedAccount<'info>,

    /// CHECK: seller's associated token account for USEDY
    #[account(mut)]
    pub seller_usedy_ata: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}