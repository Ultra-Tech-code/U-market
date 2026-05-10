use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Enums
// ─────────────────────────────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum UserType {
    Buyer,
    Seller,
    Both,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PaymentMode {
    Sol,
    Spl, // any SPL token configured on the platform (e.g. USDC)
}

// ─────────────────────────────────────────────────────────────────────────────
// Sub-structs
// ─────────────────────────────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct NegotiationTier {
    pub quantity: u64,           // minimum kg to unlock this discount
    pub discount_percentage: u8, // 1-99
}

// ─────────────────────────────────────────────────────────────────────────────
// PlatformConfig  (PDA: ["platform_config"])
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct PlatformConfig {
    pub authority: Pubkey,          // admin / upgrade authority
    pub fee_recipient: Pubkey,      // where platform fees are sent
    pub fee_percentage: u8,         // 0-99
    pub umarket_mint: Pubkey,         // reward token mint
    pub spl_payment_mint: Pubkey,   // accepted SPL payment token (e.g. USDC)
    pub product_count: u64,
    pub request_count: u64,
    pub offer_count: u64,
    pub user_count: u64,
    pub dispute_buffer: i64,        // in seconds
    pub bump: u8,
}

impl PlatformConfig {
    // 8 disc + fields
    pub const LEN: usize = 8
        + 32  // authority
        + 32  // fee_recipient
        + 1   // fee_percentage
        + 32  // umarket_mint
        + 32  // spl_payment_mint
        + 8   // product_count
        + 8   // request_count
        + 8   // offer_count
        + 8   // user_count
        + 8   // dispute_buffer
        + 1;  // bump
}

// ─────────────────────────────────────────────────────────────────────────────
// UserProfile  (PDA: ["profile", user_pubkey])
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct UserProfile {
    pub owner: Pubkey,
    pub profile_id: u64,
    pub name: String,        // max 64
    pub location: String,    // max 64
    pub mail: String,        // max 128
    pub user_type: UserType,
    pub recycled_count: u64,
    pub recycled_weight: u64,
    pub total_payout: u64,   // in lamports / token base units
    pub bump: u8,
}

impl UserProfile {
    pub const MAX_NAME: usize = 64;
    pub const MAX_LOCATION: usize = 64;
    pub const MAX_MAIL: usize = 128;

    pub const LEN: usize = 8
        + 32  // owner
        + 8   // profile_id
        + 4 + Self::MAX_NAME
        + 4 + Self::MAX_LOCATION
        + 4 + Self::MAX_MAIL
        + 1   // user_type (enum tag)
        + 8   // recycled_count
        + 8   // recycled_weight
        + 8   // total_payout
        + 1;  // bump

    pub fn can_sell(&self) -> bool {
        matches!(self.user_type, UserType::Seller | UserType::Both)
    }

    pub fn can_buy(&self) -> bool {
        matches!(self.user_type, UserType::Buyer | UserType::Both)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Product  (PDA: ["product", product_id_bytes])
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct Product {
    pub owner: Pubkey,
    pub product_id: u64,
    pub name: String,           // max 64
    pub image: String,          // max 200 (URI)
    pub location: String,       // max 64
    pub description: String,    // max 256
    pub price: u64,             // per kg, in lamports or SPL base units
    pub payment_mode: PaymentMode,
    pub total_weight: u64,      // kg available
    pub sold: u64,              // kg sold
    pub in_progress: u64,       // number of active escrows
    pub negotiation_tiers: Vec<NegotiationTier>, // max 5
    pub bump: u8,
}

impl Product {
    pub const MAX_TIERS: usize = 5;
    pub const LEN: usize = 8
        + 32        // owner
        + 8         // product_id
        + 4 + 64    // name
        + 4 + 200   // image
        + 4 + 64    // location
        + 4 + 256   // description
        + 8         // price
        + 1         // payment_mode
        + 8         // total_weight
        + 8         // sold
        + 8         // in_progress
        + 4 + (Self::MAX_TIERS * (8 + 1)) // tiers vec
        + 1;        // bump

    /// Return best discount (0-99) for a given quantity.
    pub fn best_discount(&self, quantity: u64) -> u8 {
        let mut best = 0u8;
        for tier in &self.negotiation_tiers {
            if tier.quantity > 0
                && quantity >= tier.quantity
                && tier.discount_percentage > best
            {
                best = tier.discount_percentage;
            }
        }
        best
    }

    pub fn discounted_price(&self, quantity: u64) -> u64 {
        let base = self.price * quantity;
        let discount = self.best_discount(quantity);
        if discount > 0 {
            base - (base * discount as u64) / 100
        } else {
            base
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Escrow  (PDA: ["escrow", product_id_bytes, buyer_pubkey])
// Holds the buyer's funds until they call approve_payment.
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct Escrow {
    pub buyer: Pubkey,
    pub product_id: u64,
    pub amount: u64,           // locked amount (lamports or SPL base units)
    pub amount_kg: u64,
    pub payment_mode: PaymentMode,
    pub created_at: i64,
    pub bump: u8,
}

impl Escrow {
    pub const LEN: usize = 8
        + 32  // buyer
        + 8   // product_id
        + 8   // amount
        + 8   // amount_kg
        + 1   // payment_mode
        + 8   // created_at
        + 1;  // bump
}

// ─────────────────────────────────────────────────────────────────────────────
// ProductRequest  (PDA: ["request", request_id_bytes])
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct ProductRequest {
    pub requester: Pubkey,
    pub request_id: u64,
    pub name: String,           // max 64
    pub description: String,    // max 256
    pub location: String,       // max 64
    pub max_price: u64,
    pub quantity: u64,
    pub deadline: i64,
    pub payment_mode: PaymentMode,
    pub active: bool,
    pub bump: u8,
}

impl ProductRequest {
    pub const LEN: usize = 8
        + 32        // requester
        + 8         // request_id
        + 4 + 64    // name
        + 4 + 256   // description
        + 4 + 64    // location
        + 8         // max_price
        + 8         // quantity
        + 8         // deadline
        + 1         // payment_mode
        + 1         // active
        + 1;        // bump
}

// ─────────────────────────────────────────────────────────────────────────────
// Offer  (PDA: ["offer", offer_id_bytes])
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct Offer {
    pub seller: Pubkey,
    pub request_id: u64,
    pub offer_id: u64,
    pub name: String,           // max 64
    pub image: String,          // max 200
    pub description: String,    // max 256
    pub price: u64,
    pub quantity: u64,
    pub deadline: i64,
    pub accepted: bool,
    pub delivered: bool,
    pub accepted_by: Option<Pubkey>,
    pub bump: u8,
}

impl Offer {
    pub const LEN: usize = 8
        + 32        // seller
        + 8         // request_id
        + 8         // offer_id
        + 4 + 64    // name
        + 4 + 200   // image
        + 4 + 256   // description
        + 8         // price
        + 8         // quantity
        + 8         // deadline
        + 1         // accepted
        + 1         // delivered
        + 1 + 32    // Option<Pubkey>
        + 1;        // bump
}

// ─────────────────────────────────────────────────────────────────────────────
// OfferEscrow  (PDA: ["offer_escrow", offer_id_bytes, buyer_pubkey])
// Holds funds for an accepted offer.
// ─────────────────────────────────────────────────────────────────────────────

#[account]
pub struct OfferEscrow {
    pub buyer: Pubkey,
    pub offer_id: u64,
    pub amount: u64,
    pub payment_mode: PaymentMode,
    pub created_at: i64,
    pub bump: u8,
}

impl OfferEscrow {
    pub const LEN: usize = 8
        + 32  // buyer
        + 8   // offer_id
        + 8   // amount
        + 1   // payment_mode
        + 8   // created_at
        + 1;  // bump
}