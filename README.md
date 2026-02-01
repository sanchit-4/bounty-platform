# 🛡️ Solana Gig Board (Decentralized Bounty Platform)

A trustless, decentralized freelance marketplace built on the Solana Blockchain. Connects job posters with workers using secure Smart Contract Escrows.

## 🌟 Overview

Gig Board solves the problem of trust in freelance work. Instead of relying on a middleman (like Upwork or Fiverr) to hold funds, this platform uses Solana Smart Contracts.

    Posters lock SOL into a secure Program Derived Address (PDA) when creating a gig.

    Workers apply and get selected.

    Payments are automatic and atomic upon approval—no manual transfers, no delays.

## ✨ Key Features

    🔒 On-Chain Escrow: Funds are locked in the smart contract the moment a gig is posted. The poster cannot run away with the money, and the worker cannot get paid without approval.

    🤝 Candidate Selection: Posters can view a list of applicants and select the best candidate for the job.

    🔗 Proof of Work: Workers submit a submission link (GitHub, Figma, Google Drive) directly to the blockchain.

    ⚡ Instant Payouts: Once the work is approved, the smart contract instantly transfers the locked SOL to the worker's wallet.

    🌍 Global & Permissionless: Anyone with a Solana wallet (Phantom, Solflare) can participate without registration.

## 🏗️ Tech Stack
Blockchain (Backend)

    Language: Rust

    Framework: Anchor (v0.30.1)

    Network: Solana Devnet

    RPC Provider: Helius (for high-performance connection)

Client (Frontend)

    Framework: Next.js 14 (App Router)

    Language: TypeScript

    Styling: Tailwind CSS

    Wallet Integration: @solana/wallet-adapter

    Deployment: Vercel

## ⚙️ Smart Contract Architecture

The core logic is built around a State Machine stored in the Bounty account:

    Open: Gig is posted, SOL is in escrow. Candidates can apply.

    InProgress: A worker has been selected.

    Review: Worker has submitted a link. Poster is reviewing.

    Completed: Work approved, funds released to worker.

    Cancelled: Poster cancelled before selecting a worker (refunds SOL).

Data Structure (lib.rs)
```Rust

pub struct Bounty {
    pub id: u64,              // Unique ID
    pub poster: Pubkey,       // Owner
    pub price: u64,           // Amount in Lamports
    pub description: String,  // Task details
    pub state: BountyState,   // Current status
    pub worker: Option<Pubkey>, 
    pub candidates: Vec<Pubkey>,
    pub submission: Option<String> // Link to work
}
```
## 🚀 Getting Started
Prerequisites

    Node.js (v18+)

    Rust & Cargo

    Solana CLI

    Anchor CLI

    Phantom Wallet extension

1. Installation

Clone the repository:
```Bash

git clone https://github.com/your-username/gig-board.git
cd gig-board
```
2. Backend Setup (Smart Contract)

Navigate to the anchor folder:
```Bash

cd anchor
```
Install dependencies:
```Bash

npm install
```
Build the program:
```Bash

anchor build
```
Deploy to Devnet:

    Set config to devnet: solana config set --url https://api.devnet.solana.com

    Get some devnet SOL: solana airdrop 2

    Deploy: anchor deploy

    Important: Copy the new Program ID generated after deployment.

Update lib.rs and Anchor.toml with your new Program ID if it changed.

3. Frontend Setup

Navigate to the root folder:
```Bash

cd ..
npm install
```
Environment Variables: Create a .env.local file in the root directory:
```Bash

NEXT_PUBLIC_HELIUS_RPC_URL=https://devnet.helius-rpc.com/?api-key=YOUR_HELIUS_KEY
```
Update Program ID: Open app/hooks/useGigBoardProgram.ts (or your constants file) and replace the PROGRAM_ID with the address you got from the backend deployment step.

Run the development server:
```Bash

npm run dev
```
Visit http://localhost:3000 to interact with the app.
## 🧪 How to Test (User Flow)

Since this is a marketplace, you will need two different wallets to test the full flow.

    Wallet A (Poster):

        Connect Wallet A.

        Click "Post Gig" and enter a description + price (e.g., 0.1 SOL).

        Approve transaction.

    Wallet B (Worker):

        Disconnect Wallet A and connect Wallet B.

        You will see the new gig. Click "Apply".

    Wallet A (Poster):

        Switch back to Wallet A.

        You will see Wallet B in the "Candidates" list.

        Click "Select Worker".

    Wallet B (Worker):

        Switch to Wallet B.

        Click "Submit Work" and paste a URL (e.g., a GitHub link).

    Wallet A (Poster):

        Switch to Wallet A.

        Review the link. Click "Approve & Pay".

        Result: The gig is marked Complete, and 0.1 SOL is transferred to Wallet B.

## 🛠️ Troubleshooting

    Error: 403 Forbidden on Deployment:

        This means the public RPC node is blocking your Vercel domain.

        Fix: Ensure you are using a Helius/QuickNode RPC URL in your AppWalletProvider.tsx and have whitelisted your Vercel domain in the Helius dashboard.

    Transaction Simulation Failed:

        Usually means you are trying to perform an action not allowed by the state (e.g., applying to a gig that is already "In Progress").

        Check the console for specific Anchor error codes.

🔮 Future Roadmap

    [ ] USDC Support: Upgrade from native SOL to SPL Tokens for stable payments.

    [ ] Dispute Resolution: Add an "Arbiter" role to resolve disagreements between posters and workers.

    [ ] User Profiles: Store usernames and avatars using IPFS.

    [ ] Reputation System: On-chain reputation scores for reliable workers.

📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

👨‍💻 Author

Sanchit Goyal

    GitHub: @sanchit-4

    Twitter: @Sanchitgoyal283

Built with ❤️ on Solana.