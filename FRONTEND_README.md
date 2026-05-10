# U-Market — Frontend Integration Guide

> A decentralized marketplace on Solana for agricultural commodities, supporting SOL & SPL (USDC) payments, escrow-based trade, and a request/offer negotiation system.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [On-Chain Accounts & PDAs](#on-chain-accounts--pdas)
3. [Enums & Types](#enums--types)
4. [Instructions Reference](#instructions-reference)
5. [User Flows](#user-flows)
6. [PDA Derivation Cheat-Sheet](#pda-derivation-cheat-sheet)
7. [Events](#events)
8. [Error Codes](#error-codes)
9. [Frontend Setup](#frontend-setup)
10. [UI Design Recommendations](#ui-design-recommendations)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        U-Market Program                         │
│                                                                 │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │  Platform    │  │   Profiles   │  │      Products          │ │
│  │  Config      │  │  (Buyer /    │  │  (List, Update, Buy)   │ │
│  │  (Admin)     │  │   Seller /   │  │                        │ │
│  │             │  │   Both)      │  │  ┌──────────────────┐  │ │
│  └─────────────┘  └──────────────┘  │  │  Escrow (SOL/SPL)│  │ │
│                                      │  │  → approve_pay   │  │ │
│  ┌─────────────────────────────────┐ │  └──────────────────┘  │ │
│  │  Requests → Offers → Accept    │ └────────────────────────┘ │
│  │  → OfferEscrow → confirm_delivery                          │ │
│  └─────────────────────────────────┘                           │
│                                                                 │
│  Reward: 1 UMARKET token minted on list_product, approve_payment │
└─────────────────────────────────────────────────────────────────┘
```

**Program ID:** `82wC4Yky79wYGoEhKfYcCCcZiTQaCBLxPqAU8tKKrDkF`

---

## On-Chain Accounts & PDAs

### PlatformConfig

| Field              | Type      | Description                              |
|--------------------|-----------|------------------------------------------|
| `authority`        | `Pubkey`  | Admin wallet (upgrade authority)         |
| `feeRecipient`     | `Pubkey`  | Wallet that receives platform fees       |
| `feePercentage`    | `u8`      | Platform fee 0–99%                       |
| `umarketMint`     | `Pubkey`  | UMARKET reward token mint               |
| `splPaymentMint`   | `Pubkey`  | Accepted SPL payment mint (e.g. USDC)    |
| `productCount`     | `u64`     | Auto-incrementing product counter        |
| `requestCount`     | `u64`     | Auto-incrementing request counter        |
| `offerCount`       | `u64`     | Auto-incrementing offer counter          |
| `userCount`        | `u64`     | Auto-incrementing user counter           |
| `bump`             | `u8`      | PDA bump                                 |

**Seeds:** `["platform_config"]`

### UserProfile

| Field            | Type       | Description                           |
|------------------|------------|---------------------------------------|
| `owner`          | `Pubkey`   | Wallet that owns this profile         |
| `profileId`      | `u64`      | Unique sequential ID                  |
| `name`           | `String`   | Display name (max 64 chars)           |
| `location`       | `String`   | User location (max 64 chars)          |
| `mail`           | `String`   | Contact email (max 128 chars)         |
| `userType`       | `UserType` | `Buyer`, `Seller`, or `Both`          |
| `recycledCount`  | `u64`      | Number of completed sales (seller)    |
| `recycledWeight` | `u64`      | Total kg sold                         |
| `totalPayout`    | `u64`      | Cumulative payout received            |
| `bump`           | `u8`       | PDA bump                              |

**Seeds:** `["profile", user_pubkey]`

### Product

| Field              | Type                    | Description                         |
|--------------------|-------------------------|-------------------------------------|
| `owner`            | `Pubkey`                | Seller's wallet                     |
| `productId`        | `u64`                   | Unique sequential ID                |
| `name`             | `String`                | Product name (max 64)               |
| `image`            | `String`                | Image URI (max 200)                 |
| `location`         | `String`                | Seller's location (copied at list)  |
| `description`      | `String`                | Description (max 256)               |
| `price`            | `u64`                   | Price per kg (lamports/base units)  |
| `paymentMode`      | `PaymentMode`           | `Sol` or `Spl`                      |
| `totalWeight`      | `u64`                   | Available kg                        |
| `sold`             | `u64`                   | Total kg sold                       |
| `inProgress`       | `u64`                   | Active escrow count                 |
| `negotiationTiers` | `Vec<NegotiationTier>`  | Bulk discount tiers (max 5)         |
| `bump`             | `u8`                    | PDA bump                            |

**Seeds:** `["product", product_id_le_bytes]`

### Escrow (Product Purchase)

| Field         | Type          | Description                       |
|---------------|---------------|-----------------------------------|
| `buyer`       | `Pubkey`      | Buyer's wallet                    |
| `productId`   | `u64`         | Associated product                |
| `amount`      | `u64`         | Locked payment amount             |
| `amountKg`    | `u64`         | Kg purchased                      |
| `paymentMode` | `PaymentMode` | `Sol` or `Spl`                    |
| `bump`        | `u8`          | PDA bump                          |

**Seeds:** `["escrow", product_id_le_bytes, buyer_pubkey]`

### ProductRequest

| Field         | Type          | Description                       |
|---------------|---------------|-----------------------------------|
| `requester`   | `Pubkey`      | Buyer who created the request     |
| `requestId`   | `u64`         | Unique sequential ID              |
| `name`        | `String`      | What they want (max 64)           |
| `description` | `String`      | Details (max 256)                 |
| `location`    | `String`      | Delivery location (max 64)        |
| `maxPrice`    | `u64`         | Maximum price per unit            |
| `quantity`    | `u64`         | Desired quantity (kg)             |
| `deadline`    | `i64`         | Unix timestamp deadline           |
| `paymentMode` | `PaymentMode` | `Sol` or `Spl`                    |
| `active`      | `bool`        | Whether request is still active   |
| `bump`        | `u8`          | PDA bump                          |

**Seeds:** `["request", request_id_le_bytes]`

### Offer

| Field         | Type              | Description                       |
|---------------|-------------------|-----------------------------------|
| `seller`      | `Pubkey`          | Seller who made the offer         |
| `requestId`   | `u64`             | Associated request                |
| `offerId`     | `u64`             | Unique sequential ID              |
| `name`        | `String`          | Offer product name (max 64)       |
| `image`       | `String`          | Image URI (max 200)               |
| `description` | `String`          | Description (max 256)             |
| `price`       | `u64`             | Price per unit                    |
| `quantity`    | `u64`             | Offered quantity                  |
| `deadline`    | `i64`             | Unix timestamp deadline           |
| `accepted`    | `bool`            | Whether a buyer accepted          |
| `delivered`   | `bool`            | Whether delivery is confirmed     |
| `acceptedBy`  | `Option<Pubkey>`  | Buyer who accepted                |
| `bump`        | `u8`              | PDA bump                          |

**Seeds:** `["offer", offer_id_le_bytes]`

### OfferEscrow

| Field         | Type          | Description                       |
|---------------|---------------|-----------------------------------|
| `buyer`       | `Pubkey`      | Buyer who locked funds            |
| `offerId`     | `u64`         | Associated offer                  |
| `amount`      | `u64`         | Locked payment amount             |
| `paymentMode` | `PaymentMode` | `Sol` or `Spl`                    |
| `bump`        | `u8`          | PDA bump                          |

**Seeds:** `["offer_escrow", offer_id_le_bytes, buyer_pubkey]`

---

## Enums & Types

```typescript
// User roles
type UserType = { buyer: {} } | { seller: {} } | { both: {} };

// Payment currency
type PaymentMode = { sol: {} } | { spl: {} };

// Bulk discount tier
interface NegotiationTier {
  quantity: BN;            // min kg to unlock discount
  discountPercentage: number; // 1–99
}
```

---

## Instructions Reference

### 1. `initialize`

**Who:** Platform admin (one-time setup)

| Arg              | Type   | Description        |
|------------------|--------|--------------------|
| `fee_percentage` | `u8`   | Platform fee 0–99  |

**Accounts:**
| Account           | Type      | Notes                          |
|-------------------|-----------|--------------------------------|
| `platformConfig`  | PDA init  | `["platform_config"]`          |
| `feeRecipient`    | Unchecked | Fee destination wallet         |
| `umarketMint`    | Unchecked | UMARKET reward mint           |
| `splPaymentMint`  | Unchecked | Accepted SPL mint (e.g. USDC)  |
| `authority`       | Signer    | Admin wallet                   |
| `systemProgram`   | Program   | System Program                 |

---

### 2. `updatePlatformConfig`

**Who:** Platform admin only

| Arg                  | Type              | Description               |
|----------------------|-------------------|---------------------------|
| `fee_percentage`     | `u8`              | New fee 0–99              |
| `new_fee_recipient`  | `Option<Pubkey>`  | Optionally update address |

---

### 3. `createProfile`

**Who:** Any wallet (once per wallet)

| Arg         | Type       | Description              |
|-------------|------------|--------------------------|
| `name`      | `String`   | 1–64 chars               |
| `location`  | `String`   | 1–64 chars               |
| `mail`      | `String`   | 1–128 chars              |
| `user_type` | `UserType` | `Buyer`, `Seller`, `Both`|

---

### 4. `updateProfile`

**Who:** Profile owner

| Arg        | Type     | Description  |
|------------|----------|-------------|
| `location` | `String` | 1–64 chars  |
| `mail`     | `String` | 1–128 chars |

---

### 5. `listProduct`

**Who:** Sellers (or Both)  
**Reward:** 1 UMARKET token minted to seller

| Arg                 | Type                   | Description              |
|---------------------|------------------------|--------------------------|
| `name`              | `String`               | 1–64 chars               |
| `image`             | `String`               | Image URI, 1–200 chars   |
| `description`       | `String`               | 1–256 chars              |
| `price`             | `u64`                  | Per kg (lamports/units)  |
| `weight`            | `u64`                  | Available kg             |
| `payment_mode`      | `PaymentMode`          | SOL or SPL               |
| `negotiation_tiers` | `Vec<NegotiationTier>` | Max 5 tiers              |

---

### 6. `updateProduct`

**Who:** Product owner (only when `in_progress == 0`)

Same args as `listProduct`.

---

### 7. `buyProductSol`

**Who:** Buyers  
Transfers SOL into escrow PDA.

| Arg         | Type  | Description   |
|-------------|-------|---------------|
| `amount_kg` | `u64` | Kg to buy > 0 |

---

### 8. `buyProductSpl`

**Who:** Buyers  
Transfers SPL tokens into escrow token vault.

| Arg         | Type  | Description   |
|-------------|-------|---------------|
| `amount_kg` | `u64` | Kg to buy > 0 |

---

### 9. `approvePaymentSol`

**Who:** Buyer (confirms receipt)  
Releases SOL from escrow to seller (minus fee). Closes escrow. Mints 1 UMARKET to buyer.

---

### 10. `approvePaymentSpl`

**Who:** Buyer  
Same as above but for SPL tokens.

---

### 11. `createRequest`

**Who:** Buyers  
Posts a "wanted" request for sellers to respond to.

| Arg            | Type          | Description              |
|----------------|---------------|--------------------------|
| `name`         | `String`      | 1–64 chars               |
| `description`  | `String`      | 1–256 chars              |
| `location`     | `String`      | 1–64 chars               |
| `max_price`    | `u64`         | Max price per unit       |
| `quantity`     | `u64`         | Desired kg               |
| `deadline`     | `i64`         | Future unix timestamp    |
| `payment_mode` | `PaymentMode` | SOL or SPL               |

---

### 12. `makeOffer`

**Who:** Sellers  
Responds to a request with an offer. Validated: `price <= request.max_price`, `quantity >= request.quantity`.

| Arg           | Type     | Description              |
|---------------|----------|--------------------------|
| `name`        | `String` | 1–64 chars               |
| `image`       | `String` | Image URI, 1–200 chars   |
| `description` | `String` | 1–256 chars              |
| `price`       | `u64`    | Price per unit           |
| `quantity`    | `u64`    | Offered quantity (kg)    |
| `deadline`    | `i64`    | Future unix timestamp    |

---

### 13. `acceptOfferSol`

**Who:** Buyer  
Locks `price × quantity` SOL into offer escrow.

---

### 14. `acceptOfferSpl`

**Who:** Buyer  
Locks `price × quantity` SPL into offer escrow vault.

---

### 15. `confirmDelivery`

**Who:** Buyer who accepted  
Releases offer escrow funds to seller (minus fee). Marks offer as delivered. Mints 1 UMARKET to buyer.

---

### 16. `updateUserType`

**Who:** Platform admin only  
Changes a user's role (Buyer/Seller/Both).

---

## User Flows

### Flow A: Direct Purchase (SOL)

```
Seller: createProfile(Seller) → listProduct(SOL)
Buyer:  createProfile(Buyer)  → buyProductSol(5kg) → approvePaymentSol()
```

```
Seller lists product → Buyer purchases (SOL locked in escrow)
→ Buyer confirms receipt → Escrow released to seller (fee deducted)
→ Both receive UMARKET rewards
```

### Flow B: Direct Purchase (SPL / USDC)

```
Seller: createProfile(Seller) → listProduct(SPL)
Buyer:  createProfile(Buyer)  → buyProductSpl(5kg) → approvePaymentSpl()
```

### Flow C: Request → Offer → Delivery (SOL)

```
Buyer:  createProfile(Buyer)  → createRequest(SOL)
Seller: createProfile(Seller) → makeOffer()
Buyer:  acceptOfferSol() → confirmDelivery()
```

```
Buyer posts request → Seller makes offer → Buyer accepts (SOL locked)
→ Seller delivers → Buyer confirms → Escrow released
```

### Flow D: Request → Offer → Delivery (SPL)

```
Buyer:  createRequest(SPL)
Seller: makeOffer()
Buyer:  acceptOfferSpl() → confirmDelivery()
```

> **Note:** `confirmDelivery` currently only handles SOL escrow release via lamport manipulation. SPL offer escrow release would need a dedicated `confirmDeliverySpl` instruction (not yet implemented).

---

## PDA Derivation Cheat-Sheet

```typescript
import { PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";

const PROGRAM_ID = new PublicKey("82wC4Yky79wYGoEhKfYcCCcZiTQaCBLxPqAU8tKKrDkF");

// Helper: u64 to LE buffer
const u64Le = (n: number | BN) => new BN(n).toArrayLike(Buffer, "le", 8);

// Platform Config (singleton)
const [platformConfig] = PublicKey.findProgramAddressSync(
  [Buffer.from("platform_config")], PROGRAM_ID
);

// User Profile
const [profile] = PublicKey.findProgramAddressSync(
  [Buffer.from("profile"), userPubkey.toBuffer()], PROGRAM_ID
);

// Product
const [product] = PublicKey.findProgramAddressSync(
  [Buffer.from("product"), u64Le(productId)], PROGRAM_ID
);

// Product Escrow
const [escrow] = PublicKey.findProgramAddressSync(
  [Buffer.from("escrow"), u64Le(productId), buyerPubkey.toBuffer()], PROGRAM_ID
);

// SPL Escrow Vault (for buy_product_spl)
const [escrowVault] = PublicKey.findProgramAddressSync(
  [Buffer.from("escrow_vault"), u64Le(productId), buyerPubkey.toBuffer()], PROGRAM_ID
);

// Product Request
const [request] = PublicKey.findProgramAddressSync(
  [Buffer.from("request"), u64Le(requestId)], PROGRAM_ID
);

// Offer
const [offer] = PublicKey.findProgramAddressSync(
  [Buffer.from("offer"), u64Le(offerId)], PROGRAM_ID
);

// Offer Escrow
const [offerEscrow] = PublicKey.findProgramAddressSync(
  [Buffer.from("offer_escrow"), u64Le(offerId), buyerPubkey.toBuffer()], PROGRAM_ID
);

// SPL Offer Escrow Vault (for accept_offer_spl)
const [offerEscrowVault] = PublicKey.findProgramAddressSync(
  [Buffer.from("offer_escrow_vault"), u64Le(offerId), buyerPubkey.toBuffer()], PROGRAM_ID
);
```

---

## Events

Subscribe to these program events for real-time UI updates:

| Event            | Key Fields                                       |
|------------------|--------------------------------------------------|
| `ProfileCreated` | `creator`, `name`, `profileId`, `userType`       |
| `ProfileUpdated` | `creator`, `location`, `mail`                    |
| `ProductListed`  | `lister`, `productId`, `name`, `weight`          |
| `ProductUpdated` | `owner`, `productId`, `name`, `weight`           |
| `ProductBought`  | `buyer`, `productId`, `amountKg`, `totalPaid`    |
| `PaymentApproved`| `buyer`, `productId`, `totalAmount`              |
| `RequestCreated` | `requester`, `requestId`, `name`                 |
| `OfferMade`      | `seller`, `requestId`, `offerId`                 |
| `OfferAccepted`  | `buyer`, `offerId`, `amount`                     |
| `OfferDelivered` | `seller`, `offerId`                              |
| `UserTypeUpdated`| `user`, `newType`                                |

```typescript
// Example: listening for product listings
program.addEventListener("ProductListed", (event, slot) => {
  console.log(`New product #${event.productId}: ${event.name} (${event.weight}kg)`);
});
```

---

## Error Codes

| Code                   | Hex      | Description                            |
|------------------------|----------|----------------------------------------|
| `NotRegistered`        | `0x1770` | User has no profile                    |
| `AlreadyRegistered`    | `0x1771` | Profile already exists                 |
| `EmptyString`          | `0x1772` | Required string field is empty         |
| `NotAvailable`         | `0x1773` | Product/request not available          |
| `InvalidAmount`        | `0x1774` | Amount must be > 0                     |
| `NotEnoughProduct`     | `0x1775` | Insufficient product weight            |
| `PurchaseInProgress`   | `0x1776` | Can't update product during escrow     |
| `InvalidId`            | `0x1777` | Invalid ID provided                    |
| `NotOwner`             | `0x1778` | Not the resource owner                 |
| `UnauthorizedRole`     | `0x1779` | Missing buyer/seller role              |
| `OfferExpired`         | `0x177a` | Offer deadline has passed              |
| `OfferNotAccepted`     | `0x177b` | Offer hasn't been accepted yet         |
| `RequestExpired`       | `0x177c` | Request deadline has passed            |
| `RequestInactive`      | `0x177d` | Request is no longer active            |
| `PriceTooHigh`         | `0x177e` | Offer price > request max_price        |
| `QuantityInsufficient` | `0x177f` | Offer qty < requested qty              |
| `InvalidDiscount`      | `0x1780` | Discount must be 1–99                  |
| `InvalidPrice`         | `0x1781` | Price or weight must be > 0            |
| `InvalidDeadline`      | `0x1782` | Deadline must be in the future         |
| `InvalidFee`           | `0x1783` | Fee must be 0–99                       |
| `PaymentModeMismatch`  | `0x1784` | SOL/SPL mode doesn't match             |
| `Overflow`             | `0x1785` | Arithmetic overflow                    |
| `TooManyTiers`         | `0x1786` | Max 5 negotiation tiers                |
| `OfferAlreadyAccepted` | `0x1787` | Offer was already accepted             |
| `OfferAlreadyDelivered`| `0x1788` | Delivery already confirmed             |
| `NotYourOffer`         | `0x1789` | Only the accepting buyer can confirm   |
| `NoPaymentFound`       | `0x178a` | No escrowed payment exists             |

---

## Frontend Setup

### Dependencies

```bash
npm install @coral-xyz/anchor @solana/web3.js @solana/spl-token
```

### Initializing the Client

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Connection, PublicKey, clusterApiUrl } from "@solana/web3.js";
import idl from "./idl/umarket.json";   // copy from target/idl/umarket.json
import { Umarket } from "./types/umarket"; // copy from target/types/umarket.ts

const connection = new Connection(clusterApiUrl("devnet"));
const wallet = useWallet(); // from @solana/wallet-adapter-react

const provider = new anchor.AnchorProvider(connection, wallet, {
  commitment: "confirmed",
});

const program = new Program<Umarket>(idl as any, provider);
```

### Fetching All Products

```typescript
const products = await program.account.product.all();
// Filter by seller:
const myProducts = await program.account.product.all([
  { memcmp: { offset: 8, bytes: sellerPubkey.toBase58() } },
]);
```

### Fetching All Requests

```typescript
const requests = await program.account.productRequest.all();
// Filter active only in your UI:
const activeRequests = requests.filter(r => r.account.active);
```

### Price Calculation with Discounts

```typescript
function calculatePrice(product: any, quantityKg: number): number {
  const tiers = product.negotiationTiers;
  let bestDiscount = 0;
  for (const tier of tiers) {
    if (quantityKg >= tier.quantity.toNumber() && tier.discountPercentage > bestDiscount) {
      bestDiscount = tier.discountPercentage;
    }
  }
  const base = product.price.toNumber() * quantityKg;
  return bestDiscount > 0 ? base - (base * bestDiscount) / 100 : base;
}
```

---

## UI Design Recommendations

### Suggested Page Structure

```
/                     → Landing / marketplace home
/products             → Browse all products (grid with filters)
/products/:id         → Product detail + buy action
/requests             → Browse active requests
/requests/new         → Create a new request
/requests/:id         → Request detail + offers list
/profile              → User profile dashboard
/profile/edit         → Edit profile
/dashboard            → Seller dashboard (my products, escrows, stats)
/admin                → Admin panel (platform config, user management)
```

### Key UI Components

1. **Product Card** — Show name, image, price/kg, available weight, location, payment mode badge (SOL/SPL), discount indicator
2. **Request Card** — Show name, max price, quantity, deadline countdown, payment mode, status badge
3. **Offer Card** — Show seller info, price vs. request max, quantity, deadline, accept button
4. **Escrow Status Bar** — Visual progress: `Purchased → In Transit → Delivered → Payment Released`
5. **Profile Stats** — Recycled count, total weight, total payout, UMARKET balance
6. **Negotiation Tier Table** — Editable table for sellers to set quantity/discount tiers

### State Management Tips

- **Profile existence check:** Before any action, derive and try to fetch the user's profile PDA. If it doesn't exist, redirect to profile creation.
- **Product `inProgress` counter:** When > 0, disable the "Edit Product" button and show "Escrow Active" indicator.
- **Offer `accepted` + `delivered` flags:** Drive the escrow progress UI.
- **Countdown timers:** Use `deadline` fields for requests/offers to show live countdowns.

### Wallet Integration

Use `@solana/wallet-adapter-react` with Phantom, Solflare, etc:

```tsx
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

// In your header:
<WalletMultiButton />
```

### Recommended Tech Stack

| Layer        | Recommendation                                           |
|--------------|----------------------------------------------------------|
| Framework    | Next.js 14+ (App Router) or Vite + React                |
| Wallet       | `@solana/wallet-adapter-react`                           |
| Styling      | Tailwind CSS + shadcn/ui or vanilla CSS                  |
| State        | React Query / TanStack Query for account fetching        |
| Notifications| Sonner or react-hot-toast for tx confirmations           |
| Forms        | React Hook Form + Zod validation                         |

### Design Tokens Suggestion

```css
:root {
  --color-primary: #6C5CE7;     /* Purple accent */
  --color-secondary: #00B894;   /* Green for success/SOL */
  --color-warning: #FDCB6E;     /* Yellow for pending */
  --color-danger: #E17055;      /* Red for errors */
  --color-spl: #2E86DE;         /* Blue for SPL/USDC */
  --color-bg: #0F0F23;          /* Dark background */
  --color-card: #1A1A35;        /* Card background */
  --color-text: #E2E8F0;        /* Primary text */
  --color-muted: #94A3B8;       /* Secondary text */
  --radius: 12px;
  --font-primary: 'Inter', sans-serif;
}
```

---

## Quick Start Checklist

- [ ] Copy `target/idl/umarket.json` → your frontend `src/idl/`
- [ ] Copy `target/types/umarket.ts` → your frontend `src/types/`
- [ ] Set up wallet adapter provider
- [ ] Initialize `anchor.Program` with the IDL
- [ ] Implement profile creation flow (gate all other actions behind it)
- [ ] Build product listing/browsing pages
- [ ] Implement SOL purchase + approval flow
- [ ] Add request/offer system
- [ ] Add admin dashboard (if needed)
- [ ] Subscribe to program events for real-time updates


wallet = "~/.config/solana/phantom_keypair.json"
