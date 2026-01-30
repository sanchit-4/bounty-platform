import { 
    createMint, 
    getOrCreateAssociatedTokenAccount, 
    mintTo, 
    TOKEN_PROGRAM_ID 
} from "@solana/spl-token";
import { Connection, Keypair, PublicKey, Signer } from "@solana/web3.js";

// Save this in a constant after you generate it once, so you don't make a new Mint every time.
// For now, we'll leave it empty and let the UI guide you.
export const MOCK_USDC_MINT = new PublicKey("YOUR_MINT_ADDRESS_WILL_GO_HERE"); 

export async function createMockUSDC(
    connection: Connection,
    payer: Signer, // The user's wallet adapter converted to a Signer
    recipient: PublicKey
) {
    try {
        // 1. Create a new Mint (The "Central Bank" of your fake coin)
        const mint = await createMint(
            connection,
            payer,
            payer.publicKey, // Mint Authority
            null,
            6 // Decimals (USDC has 6)
        );

        console.log("New Mint Created:", mint.toString());

        // 2. Get the user's Token Account (Pocket)
        const tokenAccount = await getOrCreateAssociatedTokenAccount(
            connection,
            payer,
            mint,
            recipient
        );

        // 3. Print 1000 USDC to the user
        await mintTo(
            connection,
            payer,
            mint,
            tokenAccount.address,
            payer,
            1000 * 1000000 // 1000 tokens * 6 decimals
        );

        return mint.toString();
    } catch (error) {
        console.error("Error creating mock USDC:", error);
        throw error;
    }
}