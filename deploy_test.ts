import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as fs from "fs";


// Make sure to match the Program ID from your local/devnet build
const PROGRAM_ID = new PublicKey("82wC4Yky79wYGoEhKfYcCCcZiTQaCBLxPqAU8tKKrDkF");

// Helper: u64 to LE buffer
const u64Le = (n: number) => new anchor.BN(n).toArrayLike(Buffer, "le", 8);

async function main() {
    console.log("Setting up Anchor provider for devnet...");

    // We are running this via ts-node from project root, so process.env.ANCHOR_WALLET should be set
    // Or we can just read the phantom keypair directly if not
    const keypairPath = "/Users/0xblackadam/.config/solana/phantom_keypair.json";
    const secretKeyString = fs.readFileSync(keypairPath, { encoding: 'utf8' });
    const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
    const walletKeypair = Keypair.fromSecretKey(secretKey);
    const wallet = new anchor.Wallet(walletKeypair);

    const connection = new anchor.web3.Connection("https://api.devnet.solana.com", "confirmed");
    const provider = new anchor.AnchorProvider(connection, wallet, {
        preflightCommitment: "confirmed",
        commitment: "confirmed",
    });
    anchor.setProvider(provider);

    // Read IDL
    const idl = JSON.parse(fs.readFileSync("./target/idl/umarket.json", "utf8"));
    const program = new Program(idl, provider) as any;

    const authority = wallet.payer;
    const feeRecipient = authority.publicKey; // Just use our own wallet as fee recipient for now

    console.log(`Using wallet: ${authority.publicKey.toBase58()}`);

    const balance = await connection.getBalance(authority.publicKey);
    console.log(`Wallet Balance: ${balance / LAMPORTS_PER_SOL} SOL`);

    // PDAs
    const [platformConfigPda] = PublicKey.findProgramAddressSync([Buffer.from("platform_config")], PROGRAM_ID);
    const [sellerProfilePda] = PublicKey.findProgramAddressSync([Buffer.from("profile"), authority.publicKey.toBuffer()], PROGRAM_ID);

    // Mints
    let usedyMint: PublicKey;
    let splPaymentMint: PublicKey;

    // Check if initialized
    let isInitialized = false;
    let configAccount: any = null;
    try {
        configAccount = await program.account.platformConfig.fetch(platformConfigPda);
        isInitialized = true;
        console.log("Platform already initialized.");
        usedyMint = configAccount.usedyMint;
        splPaymentMint = configAccount.splPaymentMint;
    } catch (e) {
        console.log("Platform not initialized yet. Initializing...");
    }

    if (!isInitialized) {
        console.log("Creating mints...");
        usedyMint = await createMint(connection, authority, platformConfigPda, null, 9);
        splPaymentMint = await createMint(connection, authority, authority.publicKey, null, 6);

        console.log("Initializing platform config...");
        await program.methods
            .initialize(5) // 5% fee
            .accounts({
                platformConfig: platformConfigPda,
                feeRecipient: feeRecipient,
                usedyMint: usedyMint,
                splPaymentMint: splPaymentMint,
                authority: authority.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        console.log("Platform initialized successfully.");
        configAccount = await program.account.platformConfig.fetch(platformConfigPda);
    }

    // Check Profile
    let hasProfile = false;
    try {
        await program.account.userProfile.fetch(sellerProfilePda);
        hasProfile = true;
        console.log("Profile already exists.");
    } catch (e) {
        console.log("Profile doesn't exist. Creating seller profile...");
    }

    if (!hasProfile) {
        await program.methods
            .createProfile("Clementina", "Bono East", "clemetina@protonmail.com", { both: {} }) // Both to allow listing and buying
            .accounts({
                profile: sellerProfilePda,
                platformConfig: platformConfigPda,
                user: authority.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();
        console.log("Profile created successfully.");
    }

    // List Product
    const nextProductId = configAccount.productCount.toNumber() + 1;
    const [productPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("product"), u64Le(nextProductId)],
        PROGRAM_ID
    );

    console.log("Creating Associated Token Account for USEDY rewards...");
    const sellerUsedyAta = await getOrCreateAssociatedTokenAccount(
        connection,
        authority,
        usedyMint,
        authority.publicKey
    );

    console.log(`Listing product with CID: QmWM6di3s6K8LXvAh4NHz7w9vwJ4Q46HBwdDqHr3DWYF4p`);
    const price = new anchor.BN(0.5 * LAMPORTS_PER_SOL);
    const weight = new anchor.BN(100);

    // image URI will be `ipfs://QmWM6di3s6K8LXvAh4NHz7w9vwJ4Q46HBwdDqHr3DWYF4p`
    const ipfsUri = "ipfs://QmWM6di3s6K8LXvAh4NHz7w9vwJ4Q46HBwdDqHr3DWYF4p";

    await program.methods
        .listProduct("Lot of Cans", ipfsUri, "Collection of recycled cans", price, weight, { sol: {} }, [])
        .accounts({
            product: productPda,
            platformConfig: platformConfigPda,
            sellerProfile: sellerProfilePda,
            usedyMint: usedyMint,
            sellerUsedyAta: sellerUsedyAta.address,
            seller: authority.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
        })
        .rpc();

    console.log(`Successfully listed product at PDA: ${productPda.toBase58()}`);

    // Fetch and show product details
    const productData = await program.account.product.fetch(productPda);
    console.log("--- Listed Product Details ---");
    console.log(`Name: ${productData.name}`);
    console.log(`Image: ${productData.image}`);
    console.log(`Price: ${productData.price.toString()} lamports`);
    console.log(`Weight: ${productData.totalWeight.toString()} kg`);
    console.log(`Owner: ${productData.owner.toBase58()}`);
}

main().catch(err => {
    console.error("Error:", err);
});
