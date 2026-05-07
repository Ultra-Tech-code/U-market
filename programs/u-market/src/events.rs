use anchor_lang::prelude::*;
use crate::state::UserType;

#[event]
pub struct ProfileCreated {
    pub creator: Pubkey,
    pub name: String,
    pub profile_id: u64,
    pub user_type: UserType,
}

#[event]
pub struct ProfileUpdated {
    pub creator: Pubkey,
    pub location: String,
    pub mail: String,
}

#[event]
pub struct ProductListed {
    pub lister: Pubkey,
    pub product_id: u64,
    pub name: String,
    pub weight: u64,
}

#[event]
pub struct ProductUpdated {
    pub owner: Pubkey,
    pub product_id: u64,
    pub name: String,
    pub weight: u64,
}

#[event]
pub struct ProductBought {
    pub buyer: Pubkey,
    pub product_id: u64,
    pub amount_kg: u64,
    pub total_paid: u64,
}

#[event]
pub struct PaymentApproved {
    pub buyer: Pubkey,
    pub product_id: u64,
    pub total_amount: u64,
}

#[event]
pub struct RequestCreated {
    pub requester: Pubkey,
    pub request_id: u64,
    pub name: String,
}

#[event]
pub struct OfferMade {
    pub seller: Pubkey,
    pub request_id: u64,
    pub offer_id: u64,
}

#[event]
pub struct OfferAccepted {
    pub buyer: Pubkey,
    pub offer_id: u64,
    pub amount: u64,
}

#[event]
pub struct OfferDelivered {
    pub seller: Pubkey,
    pub offer_id: u64,
}

#[event]
pub struct UserTypeUpdated {
    pub user: Pubkey,
    pub new_type: UserType,
}