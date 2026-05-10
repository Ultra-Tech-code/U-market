use anchor_lang::prelude::*;

#[error_code]
pub enum UmarketError {
    #[msg("User is not registered")]
    NotRegistered,

    #[msg("User is already registered")]
    AlreadyRegistered,

    #[msg("String field cannot be empty")]
    EmptyString,

    #[msg("Product / request not available")]
    NotAvailable,

    #[msg("Amount cannot be zero")]
    InvalidAmount,

    #[msg("Not enough product weight available")]
    NotEnoughProduct,

    #[msg("A purchase is currently in progress")]
    PurchaseInProgress,

    #[msg("Invalid ID provided")]
    InvalidId,

    #[msg("You are not the owner of this resource")]
    NotOwner,

    #[msg("You do not have the required role (buyer / seller)")]
    UnauthorizedRole,

    #[msg("This offer has expired")]
    OfferExpired,

    #[msg("This offer has not been accepted")]
    OfferNotAccepted,

    #[msg("This request has expired")]
    RequestExpired,

    #[msg("This request is no longer active")]
    RequestInactive,

    #[msg("Offer price exceeds buyer's maximum")]
    PriceTooHigh,

    #[msg("Offer quantity is less than requested quantity")]
    QuantityInsufficient,

    #[msg("Discount percentage must be between 1 and 99")]
    InvalidDiscount,

    #[msg("Price or weight cannot be zero")]
    InvalidPrice,

    #[msg("Deadline must be in the future")]
    InvalidDeadline,

    #[msg("Fee percentage must be between 0 and 99")]
    InvalidFee,

    #[msg("Payment mode mismatch")]
    PaymentModeMismatch,

    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Too many negotiation tiers (max 5)")]
    TooManyTiers,

    #[msg("This offer is already accepted")]
    OfferAlreadyAccepted,

    #[msg("This offer is already delivered")]
    OfferAlreadyDelivered,

    #[msg("Not your accepted offer")]
    NotYourOffer,

    #[msg("No escrow payment found")]
    NoPaymentFound,

    #[msg("Dispute buffer has not passed yet")]
    DisputeBufferNotPassed,
}