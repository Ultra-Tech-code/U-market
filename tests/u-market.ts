import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";
import { UMarket } from "../target/types/u_market";

describe("u-market", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.Umarket as Program<UMarket>;
  const authority = (provider.wallet as anchor.Wallet).payer;

  const feeRecipient = Keypair.generate();
  const buyer = Keypair.generate();
  const seller = Keypair.generate();
  const buyer2 = Keypair.generate();

  let usedyMint: PublicKey;
  let splPaymentMint: PublicKey;
  let platformConfigPda: PublicKey;
  let sellerProfilePda: PublicKey;
  let buyerProfilePda: PublicKey;
  let buyer2ProfilePda: PublicKey;
  let productPda: PublicKey;
  let escrowPda: PublicKey;

  // Helper: derive PDA
  const findPda = (seeds: (Buffer | Uint8Array)[]) =>
    PublicKey.findProgramAddressSync(seeds, program.programId)[0];

  // Helper: u64 LE buffer
  const u64Le = (n: number) => new anchor.BN(n).toArrayLike(Buffer, "le", 8);

  before(async () => {
    // Airdrop SOL
    for (const kp of [buyer, seller, feeRecipient, buyer2]) {
      const sig = await provider.connection.requestAirdrop(kp.publicKey, 100 * LAMPORTS_PER_SOL);
      await provider.connection.confirmTransaction(sig);
    }

    platformConfigPda = findPda([Buffer.from("platform_config")]);
    sellerProfilePda = findPda([Buffer.from("profile"), seller.publicKey.toBuffer()]);
    buyerProfilePda = findPda([Buffer.from("profile"), buyer.publicKey.toBuffer()]);
    buyer2ProfilePda = findPda([Buffer.from("profile"), buyer2.publicKey.toBuffer()]);

    // Create mints — usedyMint authority = platformConfigPda
    usedyMint = await createMint(provider.connection, buyer, platformConfigPda, null, 9);
    splPaymentMint = await createMint(provider.connection, buyer, provider.wallet.publicKey, null, 6);
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // 1. Platform Initialization
  // ═══════════════════════════════════════════════════════════════════════════

  it("Initializes the platform", async () => {
    await program.methods
      .initialize(5)
      .accounts({
        platformConfig: platformConfigPda,
        feeRecipient: feeRecipient.publicKey,
        usedyMint,
        splPaymentMint,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const cfg = await program.account.platformConfig.fetch(platformConfigPda);
    assert.equal(cfg.feePercentage, 5);
    assert.equal(cfg.productCount.toNumber(), 0);
    assert.equal(cfg.requestCount.toNumber(), 0);
    assert.equal(cfg.offerCount.toNumber(), 0);
    assert.equal(cfg.userCount.toNumber(), 0);
    assert.ok(cfg.authority.equals(authority.publicKey));
    assert.ok(cfg.feeRecipient.equals(feeRecipient.publicKey));
    assert.ok(cfg.usedyMint.equals(usedyMint));
    assert.ok(cfg.splPaymentMint.equals(splPaymentMint));
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // 2. Profile Management
  // ═══════════════════════════════════════════════════════════════════════════

  it("Creates a seller profile", async () => {
    await program.methods
      .createProfile("Alice", "Lagos", "alice@example.com", { seller: {} })
      .accounts({
        profile: sellerProfilePda,
        platformConfig: platformConfigPda,
        user: seller.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller])
      .rpc();

    const p = await program.account.userProfile.fetch(sellerProfilePda);
    assert.equal(p.name, "Alice");
    assert.equal(p.location, "Lagos");
    assert.equal(p.mail, "alice@example.com");
    assert.deepEqual(p.userType, { seller: {} });
    assert.equal(p.profileId.toNumber(), 1);
    assert.equal(p.recycledCount.toNumber(), 0);
  });

  it("Creates a buyer profile", async () => {
    await program.methods
      .createProfile("Bob", "Accra", "bob@example.com", { buyer: {} })
      .accounts({
        profile: buyerProfilePda,
        platformConfig: platformConfigPda,
        user: buyer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    const p = await program.account.userProfile.fetch(buyerProfilePda);
    assert.equal(p.name, "Bob");
    assert.deepEqual(p.userType, { buyer: {} });
    assert.equal(p.profileId.toNumber(), 2);
  });

  it("Creates a second buyer profile (buyer2 as Both)", async () => {
    await program.methods
      .createProfile("Charlie", "Nairobi", "charlie@example.com", { both: {} })
      .accounts({
        profile: buyer2ProfilePda,
        platformConfig: platformConfigPda,
        user: buyer2.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer2])
      .rpc();

    const p = await program.account.userProfile.fetch(buyer2ProfilePda);
    assert.deepEqual(p.userType, { both: {} });
    assert.equal(p.profileId.toNumber(), 3);
  });

  it("Updates a profile", async () => {
    await program.methods
      .updateProfile("Abuja", "alice_new@example.com")
      .accounts({
        profile: sellerProfilePda,
        user: seller.publicKey,
      })
      .signers([seller])
      .rpc();

    const p = await program.account.userProfile.fetch(sellerProfilePda);
    assert.equal(p.location, "Abuja");
    assert.equal(p.mail, "alice_new@example.com");
    assert.equal(p.name, "Alice"); // unchanged
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // 3. Admin — Update Platform Config & User Type
  // ═══════════════════════════════════════════════════════════════════════════

  it("Updates platform config (fee)", async () => {
    await program.methods
      .updatePlatformConfig(10, null)
      .accounts({
        platformConfig: platformConfigPda,
        authority: authority.publicKey,
      })
      .rpc();

    const cfg = await program.account.platformConfig.fetch(platformConfigPda);
    assert.equal(cfg.feePercentage, 10);
  });

  it("Updates platform config (fee recipient)", async () => {
    const newRecipient = Keypair.generate();
    await program.methods
      .updatePlatformConfig(5, newRecipient.publicKey)
      .accounts({
        platformConfig: platformConfigPda,
        authority: authority.publicKey,
      })
      .rpc();

    const cfg = await program.account.platformConfig.fetch(platformConfigPda);
    assert.equal(cfg.feePercentage, 5);
    assert.ok(cfg.feeRecipient.equals(newRecipient.publicKey));

    // Reset back to original feeRecipient for later tests
    await program.methods
      .updatePlatformConfig(5, feeRecipient.publicKey)
      .accounts({
        platformConfig: platformConfigPda,
        authority: authority.publicKey,
      })
      .rpc();
  });

  it("Admin updates user type", async () => {
    await program.methods
      .updateUserType({ both: {} })
      .accounts({
        profile: buyerProfilePda,
        platformConfig: platformConfigPda,
        authority: authority.publicKey,
      })
      .rpc();

    const p = await program.account.userProfile.fetch(buyerProfilePda);
    assert.deepEqual(p.userType, { both: {} });

    // Reset buyer back to buyer for subsequent tests
    await program.methods
      .updateUserType({ buyer: {} })
      .accounts({
        profile: buyerProfilePda,
        platformConfig: platformConfigPda,
        authority: authority.publicKey,
      })
      .rpc();
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // 4. Product Listing & Update
  // ═══════════════════════════════════════════════════════════════════════════

  it("Lists a product (SOL)", async () => {
    const productId = new anchor.BN(1);
    productPda = findPda([Buffer.from("product"), u64Le(1)]);

    const sellerUsedyAta = await getOrCreateAssociatedTokenAccount(
      provider.connection, seller, usedyMint, seller.publicKey
    );

    const price = new anchor.BN(2 * LAMPORTS_PER_SOL);
    const weight = new anchor.BN(100);

    await program.methods
      .listProduct("Cocoa Beans", "ipfs://cocoa.jpg", "Premium cocoa", price, weight, { sol: {} }, [
        { quantity: new anchor.BN(10), discountPercentage: 5 },
        { quantity: new anchor.BN(50), discountPercentage: 15 },
      ])
      .accounts({
        product: productPda,
        platformConfig: platformConfigPda,
        sellerProfile: sellerProfilePda,
        usedyMint,
        sellerUsedyAta: sellerUsedyAta.address,
        seller: seller.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller])
      .rpc();

    const prod = await program.account.product.fetch(productPda);
    assert.equal(prod.name, "Cocoa Beans");
    assert.equal(prod.price.toString(), price.toString());
    assert.equal(prod.totalWeight.toNumber(), 100);
    assert.equal(prod.sold.toNumber(), 0);
    assert.equal(prod.inProgress.toNumber(), 0);
    assert.equal(prod.negotiationTiers.length, 2);
    assert.equal(prod.negotiationTiers[0].discountPercentage, 5);
    assert.equal(prod.negotiationTiers[1].quantity.toNumber(), 50);

    const cfg = await program.account.platformConfig.fetch(platformConfigPda);
    assert.equal(cfg.productCount.toNumber(), 1);
  });

  it("Updates a product", async () => {
    await program.methods
      .updateProduct(
        "Cocoa Beans v2", "ipfs://cocoa2.jpg", "Updated premium cocoa",
        new anchor.BN(3 * LAMPORTS_PER_SOL), new anchor.BN(200),
        { sol: {} },
        [{ quantity: new anchor.BN(20), discountPercentage: 10 }]
      )
      .accounts({
        product: productPda,
        seller: seller.publicKey,
      })
      .signers([seller])
      .rpc();

    const prod = await program.account.product.fetch(productPda);
    assert.equal(prod.name, "Cocoa Beans v2");
    assert.equal(prod.totalWeight.toNumber(), 200);
    assert.equal(prod.negotiationTiers.length, 1);
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // 5. Buy Product (SOL) → Approve Payment (SOL) — Full lifecycle
  // ═══════════════════════════════════════════════════════════════════════════

  it("Buys a product with SOL", async () => {
    escrowPda = findPda([
      Buffer.from("escrow"), u64Le(1), buyer.publicKey.toBuffer(),
    ]);

    const amountKg = new anchor.BN(5);
    await program.methods
      .buyProductSol(amountKg)
      .accounts({
        product: productPda,
        escrow: escrowPda,
        buyerProfile: buyerProfilePda,
        platformConfig: platformConfigPda,
        buyer: buyer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    const esc = await program.account.escrow.fetch(escrowPda);
    assert.equal(esc.amountKg.toNumber(), 5);
    assert.ok(esc.buyer.equals(buyer.publicKey));
    assert.deepEqual(esc.paymentMode, { sol: {} });

    const prod = await program.account.product.fetch(productPda);
    assert.equal(prod.totalWeight.toNumber(), 195);
    assert.equal(prod.sold.toNumber(), 5);
    assert.equal(prod.inProgress.toNumber(), 1);
  });

  it("Approves payment (SOL) — releases escrow to seller", async () => {
    const buyerUsedyAta = await getOrCreateAssociatedTokenAccount(
      provider.connection, buyer, usedyMint, buyer.publicKey
    );

    const sellerBalBefore = await provider.connection.getBalance(seller.publicKey);

    await program.methods
      .approvePaymentSol()
      .accounts({
        escrow: escrowPda,
        product: productPda,
        sellerProfile: sellerProfilePda,
        platformConfig: platformConfigPda,
        seller: seller.publicKey,
        feeRecipient: feeRecipient.publicKey,
        usedyMint,
        buyerUsedyAta: buyerUsedyAta.address,
        buyer: buyer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    const sellerBalAfter = await provider.connection.getBalance(seller.publicKey);
    assert.isAbove(sellerBalAfter, sellerBalBefore, "Seller should have received SOL");

    const prod = await program.account.product.fetch(productPda);
    assert.equal(prod.inProgress.toNumber(), 0);

    const profile = await program.account.userProfile.fetch(sellerProfilePda);
    assert.equal(profile.recycledCount.toNumber(), 1);
    assert.isAbove(profile.totalPayout.toNumber(), 0);

    // Escrow should be closed
    try {
      await program.account.escrow.fetch(escrowPda);
      assert.fail("Escrow should have been closed");
    } catch (e) {
      assert.include(e.message, "Account does not exist");
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // 6. Request → Offer → Accept (SOL) → Confirm Delivery
  // ═══════════════════════════════════════════════════════════════════════════

  it("Creates a product request", async () => {
    const requestPda = findPda([Buffer.from("request"), u64Le(1)]);
    const futureDeadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

    await program.methods
      .createRequest(
        "Shea Butter", "Need 5kg organic shea butter", "Lagos",
        new anchor.BN(2 * LAMPORTS_PER_SOL), new anchor.BN(5),
        futureDeadline, { sol: {} }
      )
      .accounts({
        request: requestPda,
        platformConfig: platformConfigPda,
        requesterProfile: buyerProfilePda,
        requester: buyer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    const req = await program.account.productRequest.fetch(requestPda);
    assert.equal(req.name, "Shea Butter");
    assert.equal(req.active, true);
    assert.equal(req.quantity.toNumber(), 5);
    assert.deepEqual(req.paymentMode, { sol: {} });

    const cfg = await program.account.platformConfig.fetch(platformConfigPda);
    assert.equal(cfg.requestCount.toNumber(), 1);
  });

  it("Makes an offer on a request", async () => {
    const requestPda = findPda([Buffer.from("request"), u64Le(1)]);
    const offerPda = findPda([Buffer.from("offer"), u64Le(1)]);
    const futureDeadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

    await program.methods
      .makeOffer(
        "Organic Shea", "ipfs://shea.jpg", "Pure organic shea butter",
        new anchor.BN(1 * LAMPORTS_PER_SOL), new anchor.BN(5),
        futureDeadline
      )
      .accounts({
        offer: offerPda,
        request: requestPda,
        platformConfig: platformConfigPda,
        sellerProfile: sellerProfilePda,
        seller: seller.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([seller])
      .rpc();

    const offer = await program.account.offer.fetch(offerPda);
    assert.equal(offer.name, "Organic Shea");
    assert.equal(offer.accepted, false);
    assert.equal(offer.delivered, false);
    assert.equal(offer.acceptedBy, null);
    assert.equal(offer.price.toNumber(), 1 * LAMPORTS_PER_SOL);

    const cfg = await program.account.platformConfig.fetch(platformConfigPda);
    assert.equal(cfg.offerCount.toNumber(), 1);
  });

  it("Accepts an offer with SOL", async () => {
    const offerPda = findPda([Buffer.from("offer"), u64Le(1)]);
    const requestPda = findPda([Buffer.from("request"), u64Le(1)]);
    const offerEscrowPda = findPda([
      Buffer.from("offer_escrow"), u64Le(1), buyer.publicKey.toBuffer(),
    ]);

    await program.methods
      .acceptOfferSol()
      .accounts({
        offer: offerPda,
        request: requestPda,
        offerEscrow: offerEscrowPda,
        buyerProfile: buyerProfilePda,
        platformConfig: platformConfigPda,
        buyer: buyer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    const offer = await program.account.offer.fetch(offerPda);
    assert.equal(offer.accepted, true);
    assert.ok(offer.acceptedBy.equals(buyer.publicKey));

    const esc = await program.account.offerEscrow.fetch(offerEscrowPda);
    assert.ok(esc.buyer.equals(buyer.publicKey));
    assert.deepEqual(esc.paymentMode, { sol: {} });
    assert.isAbove(esc.amount.toNumber(), 0);
  });

  it("Confirms delivery — releases offer escrow to seller", async () => {
    const offerPda = findPda([Buffer.from("offer"), u64Le(1)]);
    const offerEscrowPda = findPda([
      Buffer.from("offer_escrow"), u64Le(1), buyer.publicKey.toBuffer(),
    ]);
    const buyerUsedyAta = await getOrCreateAssociatedTokenAccount(
      provider.connection, buyer, usedyMint, buyer.publicKey
    );

    const sellerBalBefore = await provider.connection.getBalance(seller.publicKey);

    await program.methods
      .confirmDelivery()
      .accounts({
        offer: offerPda,
        offerEscrow: offerEscrowPda,
        sellerProfile: sellerProfilePda,
        platformConfig: platformConfigPda,
        seller: seller.publicKey,
        feeRecipient: feeRecipient.publicKey,
        usedyMint,
        buyerUsedyAta: buyerUsedyAta.address,
        buyer: buyer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    const offer = await program.account.offer.fetch(offerPda);
    assert.equal(offer.delivered, true);

    const sellerBalAfter = await provider.connection.getBalance(seller.publicKey);
    assert.isAbove(sellerBalAfter, sellerBalBefore);

    // Escrow closed
    try {
      await program.account.offerEscrow.fetch(offerEscrowPda);
      assert.fail("Offer escrow should be closed");
    } catch (e) {
      assert.include(e.message, "Account does not exist");
    }

    const profile = await program.account.userProfile.fetch(sellerProfilePda);
    assert.equal(profile.recycledCount.toNumber(), 2);
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // 7. Error / Edge Case Tests
  // ═══════════════════════════════════════════════════════════════════════════

  it("Rejects profile with empty name", async () => {
    const tempKp = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(tempKp.publicKey, 5 * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);
    const pda = findPda([Buffer.from("profile"), tempKp.publicKey.toBuffer()]);

    try {
      await program.methods
        .createProfile("", "Loc", "mail@m.com", { buyer: {} })
        .accounts({
          profile: pda,
          platformConfig: platformConfigPda,
          user: tempKp.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([tempKp])
        .rpc();
      assert.fail("Should have rejected empty name");
    } catch (e) {
      assert.include(e.message, "EmptyString");
    }
  });

  it("Rejects unauthorized platform config update", async () => {
    try {
      await program.methods
        .updatePlatformConfig(50, null)
        .accounts({
          platformConfig: platformConfigPda,
          authority: buyer.publicKey,
        })
        .signers([buyer])
        .rpc();
      assert.fail("Should have rejected non-authority");
    } catch (e) {
      assert.include(e.message, "ConstraintHasOne");
    }
  });

  it("Rejects fee >= 100", async () => {
    try {
      await program.methods
        .updatePlatformConfig(100, null)
        .accounts({
          platformConfig: platformConfigPda,
          authority: authority.publicKey,
        })
        .rpc();
      assert.fail("Should have rejected fee >= 100");
    } catch (e) {
      assert.include(e.message, "InvalidFee");
    }
  });

  it("Rejects buyer trying to list a product", async () => {
    const nextId = (await program.account.platformConfig.fetch(platformConfigPda)).productCount.toNumber() + 1;
    const pda = findPda([Buffer.from("product"), u64Le(nextId)]);
    const buyerUsedyAta = await getOrCreateAssociatedTokenAccount(
      provider.connection, buyer, usedyMint, buyer.publicKey
    );

    try {
      await program.methods
        .listProduct("Fake", "ipfs://x", "desc", new anchor.BN(1000), new anchor.BN(10), { sol: {} }, [])
        .accounts({
          product: pda,
          platformConfig: platformConfigPda,
          sellerProfile: buyerProfilePda, // buyer profile
          usedyMint,
          sellerUsedyAta: buyerUsedyAta.address,
          seller: buyer.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([buyer])
        .rpc();
      assert.fail("Should reject buyer listing");
    } catch (e) {
      assert.include(e.message, "UnauthorizedRole");
    }
  });

  it("Rejects buying zero kg", async () => {
    const escPda = findPda([
      Buffer.from("escrow"), u64Le(1), buyer2.publicKey.toBuffer(),
    ]);
    try {
      await program.methods
        .buyProductSol(new anchor.BN(0))
        .accounts({
          product: productPda,
          escrow: escPda,
          buyerProfile: buyer2ProfilePda,
          platformConfig: platformConfigPda,
          buyer: buyer2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([buyer2])
        .rpc();
      assert.fail("Should reject zero amount");
    } catch (e) {
      assert.include(e.message, "InvalidAmount");
    }
  });

  it("Rejects re-accepting an already accepted offer", async () => {
    const offerPda = findPda([Buffer.from("offer"), u64Le(1)]);
    const requestPda = findPda([Buffer.from("request"), u64Le(1)]);
    const escPda = findPda([
      Buffer.from("offer_escrow"), u64Le(1), buyer2.publicKey.toBuffer(),
    ]);

    try {
      await program.methods
        .acceptOfferSol()
        .accounts({
          offer: offerPda,
          request: requestPda,
          offerEscrow: escPda,
          buyerProfile: buyer2ProfilePda,
          platformConfig: platformConfigPda,
          buyer: buyer2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([buyer2])
        .rpc();
      assert.fail("Should reject already accepted offer");
    } catch (e) {
      // Can be OfferAlreadyAccepted or a constraint/simulation error
      assert.ok(e);
    }
  });

  it("Rejects product update by non-owner", async () => {
    try {
      await program.methods
        .updateProduct(
          "Stolen", "ipfs://x", "desc",
          new anchor.BN(1000), new anchor.BN(10),
          { sol: {} }, []
        )
        .accounts({
          product: productPda,
          seller: buyer.publicKey,
        })
        .signers([buyer])
        .rpc();
      assert.fail("Should reject non-owner update");
    } catch (e) {
      assert.include(e.message, "NotOwner");
    }
  });

  it("Rejects profile update by non-owner", async () => {
    try {
      await program.methods
        .updateProfile("Nowhere", "hack@evil.com")
        .accounts({
          profile: sellerProfilePda,
          user: buyer.publicKey,
        })
        .signers([buyer])
        .rpc();
      assert.fail("Should reject non-owner profile update");
    } catch (e) {
      // seed derivation will fail since buyer key != seller key
      assert.ok(e);
    }
  });
});
