use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, UserProfile, ProductRequest, Offer};
use crate::errors::UmarketError;
use crate::events::OfferMade;

pub fn handler(
    ctx: Context<MakeOffer>,
    name: String,
    image: String,
    description: String,
    price: u64,
    quantity: u64,
    deadline: i64,
) -> Result<()> {
    require!(!name.is_empty(), UmarketError::EmptyString);
    require!(!image.is_empty(), UmarketError::EmptyString);
    require!(!description.is_empty(), UmarketError::EmptyString);
    require!(price > 0, UmarketError::InvalidPrice);
    require!(quantity > 0, UmarketError::InvalidAmount);

    let clock = Clock::get()?;
    require!(deadline > clock.unix_timestamp, UmarketError::InvalidDeadline);

    let request = &ctx.accounts.request;
    require!(request.active, UmarketError::RequestInactive);
    require!(request.deadline > clock.unix_timestamp, UmarketError::RequestExpired);
    require!(price <= request.max_price, UmarketError::PriceTooHigh);
    require!(quantity >= request.quantity, UmarketError::QuantityInsufficient);

    let config = &mut ctx.accounts.platform_config;
    config.offer_count = config.offer_count.checked_add(1).ok_or(UmarketError::Overflow)?;
    let offer_id = config.offer_count;

    let offer = &mut ctx.accounts.offer;
    offer.seller = ctx.accounts.seller.key();
    offer.request_id = request.request_id;
    offer.offer_id = offer_id;
    offer.name = name;
    offer.image = image;
    offer.description = description;
    offer.price = price;
    offer.quantity = quantity;
    offer.deadline = deadline;
    offer.accepted = false;
    offer.delivered = false;
    offer.accepted_by = None;
    offer.bump = ctx.bumps.offer;

    emit!(OfferMade {
        seller: ctx.accounts.seller.key(),
        request_id: request.request_id,
        offer_id,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MakeOffer<'info> {
    #[account(
        init,
        payer = seller,
        space = Offer::LEN,
        seeds = [b"offer", platform_config.offer_count.checked_add(1).unwrap().to_le_bytes().as_ref()],
        bump,
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        seeds = [b"request", request.request_id.to_le_bytes().as_ref()],
        bump = request.bump,
    )]
    pub request: Account<'info, ProductRequest>,

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

    #[account(mut)]
    pub seller: Signer<'info>,

    pub system_program: Program<'info, System>,
}
