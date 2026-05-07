use anchor_lang::prelude::*;
use crate::state::{Product, NegotiationTier, PaymentMode};
use crate::errors::UmarketError;
use crate::events::ProductUpdated;

pub fn handler(
    ctx: Context<UpdateProduct>,
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

    let product = &mut ctx.accounts.product;
    require!(product.in_progress == 0, UmarketError::PurchaseInProgress);

    for tier in &negotiation_tiers {
        if tier.quantity > 0 {
            require!(
                tier.discount_percentage > 0 && tier.discount_percentage < 100,
                UmarketError::InvalidDiscount
            );
        }
    }

    let product_id = product.product_id;
    product.name = name.clone();
    product.image = image;
    product.description = description;
    product.price = price;
    product.payment_mode = payment_mode;
    product.total_weight = weight;
    product.negotiation_tiers = negotiation_tiers;

    emit!(ProductUpdated {
        owner: ctx.accounts.seller.key(),
        product_id,
        name,
        weight,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(name: String, image: String, description: String, price: u64, weight: u64, payment_mode: PaymentMode, negotiation_tiers: Vec<NegotiationTier>)]
pub struct UpdateProduct<'info> {
    #[account(
        mut,
        seeds = [b"product", product.product_id.to_le_bytes().as_ref()],
        bump = product.bump,
        constraint = product.owner == seller.key() @ UmarketError::NotOwner,
    )]
    pub product: Account<'info, Product>,

    pub seller: Signer<'info>,
}