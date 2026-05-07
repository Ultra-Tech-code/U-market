use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, MintTo};
use anchor_spl::token::Mint;

declare_id!("82wC4Yky79wYGoEhKfYcCCcZiTQaCBLxPqAU8tKKrDkF");

pub mod errors;
pub mod state;
pub mod instructions;
pub mod events;

use instructions::*;

#[program]
pub mod umarket {
    use super::*;

    // ── Platform ──────────────────────────────────────────────────────────────

    pub fn initialize(
        ctx: Context<Initialize>,
        fee_percentage: u8,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, fee_percentage)
    }

    pub fn update_platform_config(
        ctx: Context<UpdatePlatformConfig>,
        fee_percentage: u8,
        new_fee_recipient: Option<Pubkey>,
    ) -> Result<()> {
        instructions::update_platform_config::handler(ctx, fee_percentage, new_fee_recipient)
    }

    // ── Profile ───────────────────────────────────────────────────────────────

    pub fn create_profile(
        ctx: Context<CreateProfile>,
        name: String,
        location: String,
        mail: String,
        user_type: UserType,
    ) -> Result<()> {
        instructions::create_profile::handler(ctx, name, location, mail, user_type)
    }

    pub fn update_profile(
        ctx: Context<UpdateProfile>,
        location: String,
        mail: String,
    ) -> Result<()> {
        instructions::update_profile::handler(ctx, location, mail)
    }

    // ── Product ───────────────────────────────────────────────────────────────

    pub fn list_product(
        ctx: Context<ListProduct>,
        name: String,
        image: String,
        description: String,
        price: u64,
        weight: u64,
        payment_mode: PaymentMode,
        negotiation_tiers: Vec<NegotiationTier>,
    ) -> Result<()> {
        instructions::list_product::handler(
            ctx, name, image, description, price, weight, payment_mode, negotiation_tiers,
        )
    }

    pub fn update_product(
        ctx: Context<UpdateProduct>,
        name: String,
        image: String,
        description: String,
        price: u64,
        weight: u64,
        payment_mode: PaymentMode,
        negotiation_tiers: Vec<NegotiationTier>,
    ) -> Result<()> {
        instructions::update_product::handler(
            ctx, name, image, description, price, weight, payment_mode, negotiation_tiers,
        )
    }

    // ── Buying (SOL) ──────────────────────────────────────────────────────────

    pub fn buy_product_sol(
        ctx: Context<BuyProductSol>,
        amount_kg: u64,
    ) -> Result<()> {
        instructions::buy_product_sol::handler(ctx, amount_kg)
    }

    pub fn approve_payment_sol(ctx: Context<ApprovePaymentSol>) -> Result<()> {
        instructions::approve_payment_sol::handler(ctx)
    }

    // ── Buying (SPL) ──────────────────────────────────────────────────────────

    pub fn buy_product_spl(
        ctx: Context<BuyProductSpl>,
        amount_kg: u64,
    ) -> Result<()> {
        instructions::buy_product_spl::handler(ctx, amount_kg)
    }

    pub fn approve_payment_spl(ctx: Context<ApprovePaymentSpl>) -> Result<()> {
        instructions::approve_payment_spl::handler(ctx)
    }

    // ── Request / Offer ───────────────────────────────────────────────────────

    pub fn create_request(
        ctx: Context<CreateRequest>,
        name: String,
        description: String,
        location: String,
        max_price: u64,
        quantity: u64,
        deadline: i64,
        payment_mode: PaymentMode,
    ) -> Result<()> {
        instructions::create_request::handler(
            ctx, name, description, location, max_price, quantity, deadline, payment_mode,
        )
    }

    pub fn make_offer(
        ctx: Context<MakeOffer>,
        name: String,
        image: String,
        description: String,
        price: u64,
        quantity: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::make_offer::handler(ctx, name, image, description, price, quantity, deadline)
    }

    pub fn accept_offer_sol(ctx: Context<AcceptOfferSol>) -> Result<()> {
        instructions::accept_offer_sol::handler(ctx)
    }

    pub fn accept_offer_spl(ctx: Context<AcceptOfferSpl>) -> Result<()> {
        instructions::accept_offer_spl::handler(ctx)
    }

    pub fn confirm_delivery(ctx: Context<ConfirmDelivery>) -> Result<()> {
        instructions::confirm_delivery::handler(ctx)
    }

    // ── Admin ─────────────────────────────────────────────────────────────────

    pub fn update_user_type(
        ctx: Context<UpdateUserType>,
        new_type: UserType,
    ) -> Result<()> {
        instructions::update_user_type::handler(ctx, new_type)
    }
}

// Re-export types used in instruction signatures so the client sees them
pub use state::{UserType, PaymentMode, NegotiationTier};