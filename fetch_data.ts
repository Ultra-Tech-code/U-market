import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import * as fs from "fs";

const PROGRAM_ID = new PublicKey("82wC4Yky79wYGoEhKfYcCCcZiTQaCBLxPqAU8tKKrDkF");

async function main() {
    console.log("Setting up Anchor provider for devnet...");
    
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

    const idl = JSON.parse(fs.readFileSync("./target/idl/umarket.json", "utf8"));
    const program = new Program(idl, provider) as any;

    const authority = wallet.publicKey;

    console.log(`\n================================`);
    console.log(`User Wallet: ${authority.toBase58()}`);
    console.log(`================================`);

    // 1. Fetch User Profile
    const [sellerProfilePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("profile"), authority.toBuffer()], 
        PROGRAM_ID
    );

    try {
        const profile = await program.account.userProfile.fetch(sellerProfilePda);
        console.log(`\n✅ USER PROFILE FOUND (PDA: ${sellerProfilePda.toBase58()})`);
        console.log(`   Name: ${profile.name}`);
        console.log(`   Location: ${profile.location}`);
        console.log(`   Email: ${profile.mail}`);
        console.log(`   Type: ${JSON.stringify(profile.userType)}`);
        console.log(`   Recycled Count: ${profile.recycledCount.toString()}`);
        console.log(`   Total Payout: ${profile.totalPayout.toString()} lamports`);
    } catch (e) {
        console.log(`\n❌ USER PROFILE NOT FOUND for this wallet.`);
    }

    // 2. Fetch All Listings
    console.log(`\n================================`);
    console.log(`ALL PRODUCT LISTINGS (DEVNET)`);
    console.log(`================================`);
    
    try {
        const allProducts = await program.account.product.all();
        
        if (allProducts.length === 0) {
            console.log("No products found on the network.");
        } else {
            console.log(`Found ${allProducts.length} product(s):\n`);
            
            allProducts.forEach((p: any, index: number) => {
                const data = p.account;
                console.log(`Product #${index + 1} | PDA: ${p.publicKey.toBase58()}`);
                console.log(`  Owner:       ${data.owner.toBase58()}`);
                console.log(`  Name:        ${data.name}`);
                console.log(`  Description: ${data.description}`);
                console.log(`  Image/CID:   ${data.image}`);
                console.log(`  Price:       ${data.price.toString()} lamports/kg`);
                console.log(`  Weight:      ${data.totalWeight.toString()} kg available`);
                console.log(`  Sold:        ${data.sold.toString()} kg`);
                console.log(`  Payment:     ${JSON.stringify(data.paymentMode)}`);
                console.log(`-----------------------------------`);
            });
        }
    } catch (e) {
        console.error("Error fetching products:", e);
    }
}

main().catch(err => {
    console.error("Error:", err);
});
