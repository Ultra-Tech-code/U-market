use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, UserProfile, ProductRequest, PaymentMode};
use crate::errors::UmarketError;
use crate::events::RequestCreated;

pub fn handler(
    ctx: Context<CreateRequest>,
    name: String,
    description: String,
    location: String,
    max_price: u64,
    quantity: u64,
    deadline: i64,
    payment_mode: PaymentMode,
) -> Result<()> {
    require!(!name.is_empty(), UmarketError::EmptyString);
    require!(!description.is_empty(), UmarketError::EmptyString);
    require!(!location.is_empty(), UmarketError::EmptyString);
    require!(max_price > 0, UmarketError::InvalidPrice);
    require!(quantity > 0, UmarketError::InvalidAmount);

    let clock = Clock::get()?;
    require!(deadline > clock.unix_timestamp, UmarketError::InvalidDeadline);

    let config = &mut ctx.accounts.platform_config;
    config.request_count = config.request_count.checked_add(1).ok_or(UmarketError::Overflow)?;
    let request_id = config.request_count;

    let request = &mut ctx.accounts.request;
    request.requester = ctx.accounts.requester.key();
    request.request_id = request_id;
    request.name = name.clone();
    request.description = description;
    request.location = location;
    request.max_price = max_price;
    request.quantity = quantity;
    request.deadline = deadline;
    request.payment_mode = payment_mode;
    request.active = true;
    request.bump = ctx.bumps.request;

    emit!(RequestCreated {
        requester: ctx.accounts.requester.key(),
        request_id,
        name,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CreateRequest<'info> {
    #[account(
        init,
        payer = requester,
        space = ProductRequest::LEN,
        seeds = [b"request", platform_config.request_count.checked_add(1).unwrap().to_le_bytes().as_ref()],
        bump,
    )]
    pub request: Account<'info, ProductRequest>,

    #[account(
        mut,
        seeds = [b"platform_config"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(
        seeds = [b"profile", requester.key().as_ref()],
        bump = requester_profile.bump,
        constraint = requester_profile.can_buy() @ UmarketError::UnauthorizedRole,
    )]
    pub requester_profile: Account<'info, UserProfile>,

    #[account(mut)]
    pub requester: Signer<'info>,

    pub system_program: Program<'info, System>,
}
