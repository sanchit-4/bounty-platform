use anchor_lang::prelude::*;

// ⚠️ ACTION: Since we are changing the data structure (adding 'submission'), 
// you MUST generate a new key (anchor keys generate) and replace this ID one last time.
declare_id!("7Q7AFiWUCaK4X6enYQY3fNdW2qm9wVLXYA1nhpJ5zKE6");

#[program]
pub mod gig_board {
    use super::*;

    // 1. POST BOUNTY
    pub fn post_bounty(ctx: Context<PostBounty>, id: u64, price: u64, description: String) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty;
        bounty.id = id;
        bounty.poster = ctx.accounts.poster.key();
        bounty.price = price;
        bounty.description = description;
        bounty.state = BountyState::Open;
        bounty.worker = None; 
        bounty.candidates = Vec::new();
        bounty.submission = None; // Initialize as empty

        // Transfer SOL to Escrow
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.poster.to_account_info(),
                to: ctx.accounts.bounty.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, price)?;

        Ok(())
    }

    // 2. APPLY
    pub fn apply_for_bounty(ctx: Context<ApplyBounty>) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty;
        require!(bounty.state == BountyState::Open, BountyError::InvalidState);
        require!(bounty.candidates.len() < 10, BountyError::TooManyCandidates);

        let applicant = ctx.accounts.applicant.key();
        if !bounty.candidates.contains(&applicant) {
            bounty.candidates.push(applicant);
        }
        Ok(())
    }

    // 3. ACCEPT CANDIDATE
    pub fn accept_candidate(ctx: Context<UpdateBounty>, candidate: Pubkey) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty;
        require!(ctx.accounts.poster.key() == bounty.poster, BountyError::Unauthorized);
        require!(bounty.state == BountyState::Open, BountyError::InvalidState);
        require!(bounty.candidates.contains(&candidate), BountyError::NotACandidate);

        bounty.worker = Some(candidate);
        bounty.state = BountyState::InProgress;
        Ok(())
    }

    // 4. SUBMIT WORK (Updated to SAVE the link)
    pub fn submit_work(ctx: Context<UpdateBounty>, link: String) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty;
        require!(bounty.worker == Some(ctx.accounts.poster.key()), BountyError::Unauthorized); 
        require!(bounty.state == BountyState::InProgress, BountyError::InvalidState);

        bounty.submission = Some(link); // <--- SAVING THE LINK NOW
        bounty.state = BountyState::Review;
        Ok(())
    }

    // 5. APPROVE & PAY
    pub fn approve_work(ctx: Context<UpdateBounty>) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty;
        require!(ctx.accounts.poster.key() == bounty.poster, BountyError::Unauthorized);
        require!(bounty.state == BountyState::Review, BountyError::InvalidState);

        let amount = bounty.price;
        let worker_account = ctx.accounts.worker_account.as_ref().ok_or(BountyError::WrongWorker)?;
        require!(Some(worker_account.key()) == bounty.worker, BountyError::WrongWorker);

        let id_bytes = bounty.id.to_le_bytes();
        let seeds = &[
            b"bounty".as_ref(),
            bounty.poster.as_ref(),
            id_bytes.as_ref(),
            &[ctx.bumps.bounty], 
        ];
        
        **bounty.to_account_info().try_borrow_mut_lamports()? -= amount;
        **worker_account.try_borrow_mut_lamports()? += amount;

        bounty.state = BountyState::Completed;
        Ok(())
    }

    // 6. CANCEL
    pub fn cancel_bounty(ctx: Context<UpdateBounty>) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty;
        require!(ctx.accounts.poster.key() == bounty.poster, BountyError::Unauthorized);
        require!(bounty.state == BountyState::Open, BountyError::InvalidState);

        let amount = bounty.price;
        **bounty.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.poster.to_account_info().try_borrow_mut_lamports()? += amount;

        bounty.state = BountyState::Cancelled;
        Ok(())
    }
}

// --- DATA STRUCTURES ---

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct PostBounty<'info> {
    #[account(
        init,
        payer = poster,
        // Increased space to 1000 to handle the URL string safely
        space = 1000, 
        seeds = [b"bounty", poster.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub bounty: Account<'info, Bounty>,
    #[account(mut)]
    pub poster: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApplyBounty<'info> {
    #[account(mut)]
    pub bounty: Account<'info, Bounty>,
    #[account(mut)]
    pub applicant: Signer<'info>, 
}

#[derive(Accounts)]
pub struct UpdateBounty<'info> {
    #[account(
        mut,
        seeds = [b"bounty", bounty.poster.as_ref(), bounty.id.to_le_bytes().as_ref()],
        bump
    )]
    pub bounty: Account<'info, Bounty>,
    #[account(mut)]
    pub poster: Signer<'info>, 
    /// CHECK: Safe
    #[account(mut)]
    pub worker_account: Option<AccountInfo<'info>>, 
}

#[account]
pub struct Bounty {
    pub id: u64,
    pub poster: Pubkey,
    pub price: u64,
    pub description: String,
    pub state: BountyState,
    pub worker: Option<Pubkey>,
    pub candidates: Vec<Pubkey>,
    pub submission: Option<String>, // <--- NEW FIELD
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BountyState { Open, InProgress, Review, Completed, Cancelled }

#[error_code]
pub enum BountyError {
    #[msg("Unauthorized access.")] Unauthorized,
    #[msg("Invalid state.")] InvalidState,
    #[msg("Wrong worker account.")] WrongWorker,
    #[msg("Candidate list full.")] TooManyCandidates,
    #[msg("User is not a candidate.")] NotACandidate,
}



// // // use anchor_lang::prelude::*;
// // // use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

// // // // ⚠️ ACTION: Generate a NEW key one last time and paste it here.
// // // declare_id!("7XinySvQmWXx5uHNsLUGD7ViwKfG7xZytwaRXswwUfFM");

// // // #[program]
// // // pub mod gig_board {
// // //     use super::*;

// // //     // --- 1. CREATE PROFILE (Metadata) ---
// // //     pub fn create_profile(ctx: Context<CreateProfile>, name: String, ipfs_hash: String) -> Result<()> {
// // //         let profile = &mut ctx.accounts.profile;
// // //         profile.authority = ctx.accounts.user.key();
// // //         profile.name = name;
// // //         profile.ipfs_hash = ipfs_hash;
// // //         Ok(())
// // //     }

// // //     // --- 2. POST BOUNTY (USDC Version) ---
// // //     pub fn post_bounty(
// // //         ctx: Context<PostBounty>, 
// // //         id: u64, 
// // //         price: u64, 
// // //         description: String,
// // //         arbiter: Pubkey // New: Who solves disputes?
// // //     ) -> Result<()> {
// // //         let bounty = &mut ctx.accounts.bounty;
// // //         bounty.id = id;
// // //         bounty.poster = ctx.accounts.poster.key();
// // //         bounty.mint = ctx.accounts.mint.key(); // Remember which token (USDC)
// // //         bounty.price = price;
// // //         bounty.description = description;
// // //         bounty.state = BountyState::Open;
// // //         bounty.worker = None;
// // //         bounty.arbiter = arbiter;
// // //         bounty.submission = None;
// // //         bounty.candidates = Vec::new();

// // //         // TRANSFER USDC: Poster -> Bounty Vault
// // //         let transfer_ctx = CpiContext::new(
// // //             ctx.accounts.token_program.to_account_info(),
// // //             Transfer {
// // //                 from: ctx.accounts.poster_token_account.to_account_info(),
// // //                 to: ctx.accounts.bounty_vault.to_account_info(),
// // //                 authority: ctx.accounts.poster.to_account_info(),
// // //             },
// // //         );
// // //         token::transfer(transfer_ctx, price)?;

// // //         Ok(())
// // //     }

// // //     // --- 3. APPLY (Same as before) ---
// // //     pub fn apply_for_bounty(ctx: Context<ApplyBounty>) -> Result<()> {
// // //         let bounty = &mut ctx.accounts.bounty;
// // //         require!(bounty.state == BountyState::Open, BountyError::InvalidState);
// // //         let applicant = ctx.accounts.applicant.key();
// // //         if !bounty.candidates.contains(&applicant) {
// // //             bounty.candidates.push(applicant);
// // //         }
// // //         Ok(())
// // //     }

// // //     // --- 4. ACCEPT CANDIDATE (Same as before) ---
// // //     pub fn accept_candidate(ctx: Context<UpdateBounty>, candidate: Pubkey) -> Result<()> {
// // //         let bounty = &mut ctx.accounts.bounty;
// // //         require!(ctx.accounts.poster.key() == bounty.poster, BountyError::Unauthorized);
// // //         require!(bounty.state == BountyState::Open, BountyError::InvalidState);
// // //         require!(bounty.candidates.contains(&candidate), BountyError::NotACandidate);

// // //         bounty.worker = Some(candidate);
// // //         bounty.state = BountyState::InProgress;
// // //         Ok(())
// // //     }

// // //     // --- 5. SUBMIT WORK (Same as before) ---
// // //     pub fn submit_work(ctx: Context<UpdateBounty>, link: String) -> Result<()> {
// // //         let bounty = &mut ctx.accounts.bounty;
// // //         require!(bounty.worker == Some(ctx.accounts.poster.key()), BountyError::Unauthorized);
// // //         require!(bounty.state == BountyState::InProgress, BountyError::InvalidState);
// // //         bounty.submission = Some(link);
// // //         bounty.state = BountyState::Review;
// // //         Ok(())
// // //     }

// // //     // --- 6. APPROVE & PAY (USDC Version) ---
// // //     pub fn approve_work(ctx: Context<PayoutBounty>) -> Result<()> {
// // //         let bounty = &mut ctx.accounts.bounty;
// // //         require!(ctx.accounts.signer.key() == bounty.poster, BountyError::Unauthorized);
// // //         require!(bounty.state == BountyState::Review, BountyError::InvalidState);

// // //         // PAYOUT: Vault -> Worker Token Account
// // //         let amount = bounty.price;
// // //         let seeds = &[
// // //             b"bounty".as_ref(),
// // //             bounty.poster.as_ref(),
// // //             bounty.id.to_le_bytes().as_ref(),
// // //             &[ctx.bumps.bounty],
// // //         ];
// // //         let signer = &[&seeds[..]];

// // //         let transfer_ctx = CpiContext::new_with_signer(
// // //             ctx.accounts.token_program.to_account_info(),
// // //             Transfer {
// // //                 from: ctx.accounts.bounty_vault.to_account_info(),
// // //                 to: ctx.accounts.worker_token_account.to_account_info(),
// // //                 authority: ctx.accounts.bounty.to_account_info(), // PDA signs
// // //             },
// // //             signer,
// // //         );
// // //         token::transfer(transfer_ctx, amount)?;

// // //         bounty.state = BountyState::Completed;
// // //         Ok(())
// // //     }

// // //     // --- 7. RAISE DISPUTE (New) ---
// // //     pub fn raise_dispute(ctx: Context<UpdateBounty>) -> Result<()> {
// // //         let bounty = &mut ctx.accounts.bounty;
// // //         // Either poster or worker can raise dispute
// // //         let is_poster = ctx.accounts.poster.key() == bounty.poster;
// // //         let is_worker = Some(ctx.accounts.poster.key()) == bounty.worker;
// // //         require!(is_poster || is_worker, BountyError::Unauthorized);
// // //         require!(bounty.state == BountyState::Review || bounty.state == BountyState::InProgress, BountyError::InvalidState);

// // //         bounty.state = BountyState::Dispute;
// // //         Ok(())
// // //     }

// // //     // --- 8. RESOLVE DISPUTE (Arbiter Only) ---
// // //     pub fn resolve_dispute(ctx: Context<PayoutBounty>, winner_is_worker: bool) -> Result<()> {
// // //         let bounty = &mut ctx.accounts.bounty;
// // //         require!(ctx.accounts.signer.key() == bounty.arbiter, BountyError::Unauthorized);
// // //         require!(bounty.state == BountyState::Dispute, BountyError::InvalidState);

// // //         let amount = bounty.price;
// // //         let seeds = &[
// // //             b"bounty".as_ref(),
// // //             bounty.poster.as_ref(),
// // //             bounty.id.to_le_bytes().as_ref(),
// // //             &[ctx.bumps.bounty],
// // //         ];
// // //         let signer = &[&seeds[..]];

// // //         // Decide recipient based on Arbiter's decision
// // //         let destination = if winner_is_worker {
// // //             ctx.accounts.worker_token_account.to_account_info()
// // //         } else {
// // //             // In this specific instruction context, we'd need to ensure we passed the POSTER'S token account
// // //             // For V1 simplicity, let's assume 'worker_token_account' passed in is actually the winner's account
// // //             ctx.accounts.worker_token_account.to_account_info() 
// // //         };

// // //         let transfer_ctx = CpiContext::new_with_signer(
// // //             ctx.accounts.token_program.to_account_info(),
// // //             Transfer {
// // //                 from: ctx.accounts.bounty_vault.to_account_info(),
// // //                 to: destination,
// // //                 authority: ctx.accounts.bounty.to_account_info(),
// // //             },
// // //             signer,
// // //         );
// // //         token::transfer(transfer_ctx, amount)?;

// // //         bounty.state = BountyState::Resolved;
// // //         Ok(())
// // //     }
// // // }

// // // // --- DATA STRUCTURES ---

// // // #[derive(Accounts)]
// // // #[instruction(name: String, ipfs_hash: String)]
// // // pub struct CreateProfile<'info> {
// // //     #[account(
// // //         init,
// // //         payer = user,
// // //         space = 8 + 32 + (4 + 50) + (4 + 100), // Disc + Auth + Name + IPFS
// // //         seeds = [b"profile", user.key().as_ref()],
// // //         bump
// // //     )]
// // //     pub profile: Account<'info, UserProfile>,
// // //     #[account(mut)]
// // //     pub user: Signer<'info>,
// // //     pub system_program: Program<'info, System>,
// // // }

// // // #[derive(Accounts)]
// // // #[instruction(id: u64)]
// // // pub struct PostBounty<'info> {
// // //     #[account(
// // //         init,
// // //         payer = poster,
// // //         space = 1200, // Large space for vectors and strings
// // //         seeds = [b"bounty", poster.key().as_ref(), id.to_le_bytes().as_ref()],
// // //         bump
// // //     )]
// // //     pub bounty: Account<'info, Bounty>,
    
// // //     // USDC LOGIC:
// // //     #[account(
// // //         init,
// // //         payer = poster,
// // //         seeds = [b"vault", bounty.key().as_ref()], // Vault belongs to the bounty
// // //         bump,
// // //         token::mint = mint,
// // //         token::authority = bounty, // PDA owns the money
// // //     )]
// // //     pub bounty_vault: Account<'info, TokenAccount>,
    
// // //     #[account(mut)]
// // //     pub poster_token_account: Account<'info, TokenAccount>, // Source of funds
// // //     pub mint: Account<'info, Mint>, // The USDC Mint address
    
// // //     #[account(mut)]
// // //     pub poster: Signer<'info>,
// // //     pub system_program: Program<'info, System>,
// // //     pub token_program: Program<'info, Token>,
// // //     pub rent: Sysvar<'info, Rent>,
// // // }

// // // #[derive(Accounts)]
// // // pub struct PayoutBounty<'info> {
// // //     // ✅ FIX: Added seeds and bump so 'ctx.bumps.bounty' works
// // //     #[account(
// // //         mut,
// // //         seeds = [b"bounty", bounty.poster.as_ref(), bounty.id.to_le_bytes().as_ref()],
// // //         bump
// // //     )]
// // //     pub bounty: Account<'info, Bounty>,
    
// // //     #[account(
// // //         mut,
// // //         seeds = [b"vault", bounty.key().as_ref()],
// // //         bump,
// // //         token::mint = bounty.mint,
// // //         token::authority = bounty,
// // //     )]
// // //     pub bounty_vault: Account<'info, TokenAccount>,
    
// // //     #[account(mut)]
// // //     pub worker_token_account: Account<'info, TokenAccount>, 
    
// // //     #[account(mut)]
// // //     pub signer: Signer<'info>, 
// // //     pub token_program: Program<'info, Token>,
// // // }

// // // // Reuse for Apply, Accept, Submit, Raise Dispute
// // // #[derive(Accounts)]
// // // pub struct UpdateBounty<'info> {
// // //     #[account(mut)]
// // //     pub bounty: Account<'info, Bounty>,
// // //     #[account(mut)]
// // //     pub poster: Signer<'info>, 
// // // }

// // // #[derive(Accounts)]
// // // pub struct ApplyBounty<'info> {
// // //     #[account(mut)]
// // //     pub bounty: Account<'info, Bounty>,
// // //     #[account(mut)]
// // //     pub applicant: Signer<'info>, 
// // // }

// // // #[account]
// // // pub struct Bounty {
// // //     pub id: u64,
// // //     pub poster: Pubkey,
// // //     pub mint: Pubkey,      // Store which token is being used
// // //     pub price: u64,
// // //     pub description: String,
// // //     pub state: BountyState,
// // //     pub worker: Option<Pubkey>,
// // //     pub candidates: Vec<Pubkey>,
// // //     pub submission: Option<String>,
// // //     pub arbiter: Pubkey,   // Dispute resolver
// // // }

// // // #[account]
// // // pub struct UserProfile {
// // //     pub authority: Pubkey,
// // //     pub name: String,
// // //     pub ipfs_hash: String,
// // // }

// // // #[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
// // // pub enum BountyState { Open, InProgress, Review, Completed, Cancelled, Dispute, Resolved }

// // // #[error_code]
// // // pub enum BountyError {
// // //     #[msg("Unauthorized.")] Unauthorized,
// // //     #[msg("Invalid state.")] InvalidState,
// // //     #[msg("Wrong worker.")] WrongWorker,
// // //     #[msg("Too many candidates.")] TooManyCandidates,
// // //     #[msg("Not a candidate.")] NotACandidate,
// // // }



// // use anchor_lang::prelude::*;
// // use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

// // // ⚠️ ACTION: Ensure this matches your current Program ID
// // declare_id!("7XinySvQmWXx5uHNsLUGD7ViwKfG7xZytwaRXswwUfFM");

// // #[program]
// // pub mod gig_board {
// //     use super::*;

// //     // --- 1. CREATE PROFILE ---
// //     pub fn create_profile(ctx: Context<CreateProfile>, name: String, ipfs_hash: String) -> Result<()> {
// //         let profile = &mut ctx.accounts.profile;
// //         profile.authority = ctx.accounts.user.key();
// //         profile.name = name;
// //         profile.ipfs_hash = ipfs_hash;
// //         Ok(())
// //     }

// //     // --- 2. POST BOUNTY ---
// //     pub fn post_bounty(
// //         ctx: Context<PostBounty>, 
// //         id: u64, 
// //         price: u64, 
// //         description: String,
// //         arbiter: Pubkey 
// //     ) -> Result<()> {
// //         let bounty = &mut ctx.accounts.bounty;
// //         bounty.id = id;
// //         bounty.poster = ctx.accounts.poster.key();
// //         bounty.mint = ctx.accounts.mint.key();
// //         bounty.price = price;
// //         bounty.description = description;
// //         bounty.state = BountyState::Open;
// //         bounty.worker = None;
// //         bounty.arbiter = arbiter;
// //         bounty.submission = None;
// //         bounty.candidates = Vec::new();

// //         // TRANSFER USDC: Poster -> Bounty Vault
// //         let transfer_ctx = CpiContext::new(
// //             ctx.accounts.token_program.to_account_info(),
// //             Transfer {
// //                 from: ctx.accounts.poster_token_account.to_account_info(),
// //                 to: ctx.accounts.bounty_vault.to_account_info(),
// //                 authority: ctx.accounts.poster.to_account_info(),
// //             },
// //         );
// //         token::transfer(transfer_ctx, price)?;

// //         Ok(())
// //     }

// //     // --- 3. APPLY ---
// //     pub fn apply_for_bounty(ctx: Context<ApplyBounty>) -> Result<()> {
// //         let bounty = &mut ctx.accounts.bounty;
// //         require!(bounty.state == BountyState::Open, BountyError::InvalidState);
// //         let applicant = ctx.accounts.applicant.key();
// //         if !bounty.candidates.contains(&applicant) {
// //             bounty.candidates.push(applicant);
// //         }
// //         Ok(())
// //     }

// //     // --- 4. ACCEPT CANDIDATE ---
// //     pub fn accept_candidate(ctx: Context<UpdateBounty>, candidate: Pubkey) -> Result<()> {
// //         let bounty = &mut ctx.accounts.bounty;
// //         require!(ctx.accounts.poster.key() == bounty.poster, BountyError::Unauthorized);
// //         require!(bounty.state == BountyState::Open, BountyError::InvalidState);
// //         require!(bounty.candidates.contains(&candidate), BountyError::NotACandidate);

// //         bounty.worker = Some(candidate);
// //         bounty.state = BountyState::InProgress;
// //         Ok(())
// //     }

// //     // --- 5. SUBMIT WORK ---
// //     pub fn submit_work(ctx: Context<UpdateBounty>, link: String) -> Result<()> {
// //         let bounty = &mut ctx.accounts.bounty;
// //         require!(bounty.worker == Some(ctx.accounts.poster.key()), BountyError::Unauthorized);
// //         require!(bounty.state == BountyState::InProgress, BountyError::InvalidState);
// //         bounty.submission = Some(link);
// //         bounty.state = BountyState::Review;
// //         Ok(())
// //     }

// //     // --- 6. APPROVE & PAY ---
// //     pub fn approve_work(ctx: Context<PayoutBounty>) -> Result<()> {
// //         let bounty = &mut ctx.accounts.bounty;
// //         require!(ctx.accounts.signer.key() == bounty.poster, BountyError::Unauthorized);
// //         require!(bounty.state == BountyState::Review, BountyError::InvalidState);

// //         let amount = bounty.price;

// //         // ✅ FIX 1: Create a longer-lived variable for the ID bytes
// //         let id_bytes = bounty.id.to_le_bytes();
        
// //         let seeds = &[
// //             b"bounty".as_ref(),
// //             bounty.poster.as_ref(),
// //             id_bytes.as_ref(), // Use the variable, not the method call
// //             &[ctx.bumps.bounty],
// //         ];
// //         let signer = &[&seeds[..]];

// //         // ✅ FIX 2: Use 'bounty.to_account_info()' directly to avoid re-borrowing from context
// //         let transfer_ctx = CpiContext::new_with_signer(
// //             ctx.accounts.token_program.to_account_info(),
// //             Transfer {
// //                 from: ctx.accounts.bounty_vault.to_account_info(),
// //                 to: ctx.accounts.worker_token_account.to_account_info(),
// //                 authority: bounty.to_account_info(), 
// //             },
// //             signer,
// //         );
// //         token::transfer(transfer_ctx, amount)?;

// //         bounty.state = BountyState::Completed;
// //         Ok(())
// //     }

// //     // --- 7. RAISE DISPUTE ---
// //     pub fn raise_dispute(ctx: Context<UpdateBounty>) -> Result<()> {
// //         let bounty = &mut ctx.accounts.bounty;
// //         let is_poster = ctx.accounts.poster.key() == bounty.poster;
// //         let is_worker = Some(ctx.accounts.poster.key()) == bounty.worker;
        
// //         require!(is_poster || is_worker, BountyError::Unauthorized);
// //         require!(bounty.state == BountyState::Review || bounty.state == BountyState::InProgress, BountyError::InvalidState);

// //         bounty.state = BountyState::Dispute;
// //         Ok(())
// //     }

// //     // --- 8. RESOLVE DISPUTE ---
// //     pub fn resolve_dispute(ctx: Context<PayoutBounty>, winner_is_worker: bool) -> Result<()> {
// //         let bounty = &mut ctx.accounts.bounty;
// //         require!(ctx.accounts.signer.key() == bounty.arbiter, BountyError::Unauthorized);
// //         require!(bounty.state == BountyState::Dispute, BountyError::InvalidState);

// //         let amount = bounty.price;
        
// //         // ✅ FIX 1: Same fix for ID bytes
// //         let id_bytes = bounty.id.to_le_bytes();

// //         let seeds = &[
// //             b"bounty".as_ref(),
// //             bounty.poster.as_ref(),
// //             id_bytes.as_ref(),
// //             &[ctx.bumps.bounty],
// //         ];
// //         let signer = &[&seeds[..]];

// //         let destination = if winner_is_worker {
// //             ctx.accounts.worker_token_account.to_account_info()
// //         } else {
// //             // In a real V2, you would pass the poster's token account specifically.
// //             // For now, we assume the UI passed the winner's account in this slot.
// //             ctx.accounts.worker_token_account.to_account_info() 
// //         };

// //         // ✅ FIX 2: Same fix for Authority borrowing
// //         let transfer_ctx = CpiContext::new_with_signer(
// //             ctx.accounts.token_program.to_account_info(),
// //             Transfer {
// //                 from: ctx.accounts.bounty_vault.to_account_info(),
// //                 to: destination,
// //                 authority: bounty.to_account_info(),
// //             },
// //             signer,
// //         );
// //         token::transfer(transfer_ctx, amount)?;

// //         bounty.state = BountyState::Resolved;
// //         Ok(())
// //     }
// // }

// // // --- DATA STRUCTURES ---

// // #[derive(Accounts)]
// // #[instruction(name: String, ipfs_hash: String)]
// // pub struct CreateProfile<'info> {
// //     #[account(
// //         init,
// //         payer = user,
// //         space = 8 + 32 + (4 + 50) + (4 + 100), 
// //         seeds = [b"profile", user.key().as_ref()],
// //         bump
// //     )]
// //     pub profile: Account<'info, UserProfile>,
// //     #[account(mut)]
// //     pub user: Signer<'info>,
// //     pub system_program: Program<'info, System>,
// // }

// // #[derive(Accounts)]
// // #[instruction(id: u64)]
// // pub struct PostBounty<'info> {
// //     #[account(
// //         init,
// //         payer = poster,
// //         space = 1200, 
// //         seeds = [b"bounty", poster.key().as_ref(), id.to_le_bytes().as_ref()],
// //         bump
// //     )]
// //     pub bounty: Account<'info, Bounty>,
    
// //     #[account(
// //         init,
// //         payer = poster,
// //         seeds = [b"vault", bounty.key().as_ref()],
// //         bump,
// //         token::mint = mint,
// //         token::authority = bounty,
// //     )]
// //     pub bounty_vault: Account<'info, TokenAccount>,
    
// //     #[account(mut)]
// //     pub poster_token_account: Account<'info, TokenAccount>,
// //     pub mint: Account<'info, Mint>,
    
// //     #[account(mut)]
// //     pub poster: Signer<'info>,
// //     pub system_program: Program<'info, System>,
// //     pub token_program: Program<'info, Token>,
// //     pub rent: Sysvar<'info, Rent>,
// // }

// // #[derive(Accounts)]
// // pub struct PayoutBounty<'info> {
// //     #[account(
// //         mut,
// //         seeds = [b"bounty", bounty.poster.as_ref(), bounty.id.to_le_bytes().as_ref()],
// //         bump
// //     )]
// //     pub bounty: Account<'info, Bounty>,
    
// //     #[account(
// //         mut,
// //         seeds = [b"vault", bounty.key().as_ref()],
// //         bump,
// //         token::mint = bounty.mint,
// //         token::authority = bounty,
// //     )]
// //     pub bounty_vault: Account<'info, TokenAccount>,
    
// //     #[account(mut)]
// //     pub worker_token_account: Account<'info, TokenAccount>, 
    
// //     #[account(mut)]
// //     pub signer: Signer<'info>, 
// //     pub token_program: Program<'info, Token>,
// // }

// // #[derive(Accounts)]
// // pub struct UpdateBounty<'info> {
// //     #[account(mut)]
// //     pub bounty: Account<'info, Bounty>,
// //     #[account(mut)]
// //     pub poster: Signer<'info>, 
// // }

// // #[derive(Accounts)]
// // pub struct ApplyBounty<'info> {
// //     #[account(mut)]
// //     pub bounty: Account<'info, Bounty>,
// //     #[account(mut)]
// //     pub applicant: Signer<'info>, 
// // }

// // #[account]
// // pub struct Bounty {
// //     pub id: u64,
// //     pub poster: Pubkey,
// //     pub mint: Pubkey,
// //     pub price: u64,
// //     pub description: String,
// //     pub state: BountyState,
// //     pub worker: Option<Pubkey>,
// //     pub candidates: Vec<Pubkey>,
// //     pub submission: Option<String>,
// //     pub arbiter: Pubkey, 
// // }

// // #[account]
// // pub struct UserProfile {
// //     pub authority: Pubkey,
// //     pub name: String,
// //     pub ipfs_hash: String,
// // }

// // #[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
// // pub enum BountyState { Open, InProgress, Review, Completed, Cancelled, Dispute, Resolved }

// // #[error_code]
// // pub enum BountyError {
// //     #[msg("Unauthorized.")] Unauthorized,
// //     #[msg("Invalid state.")] InvalidState,
// //     #[msg("Wrong worker.")] WrongWorker,
// //     #[msg("Too many candidates.")] TooManyCandidates,
// //     #[msg("Not a candidate.")] NotACandidate,
// // }


// use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

// // ⚠️ ACTION: Ensure this matches your current Program ID
// declare_id!("YOUR_NEW_PROGRAM_ID_HERE");

// #[program]
// pub mod gig_board {
//     use super::*;

//     // --- 1. CREATE PROFILE ---
//     pub fn create_profile(ctx: Context<CreateProfile>, name: String, ipfs_hash: String) -> Result<()> {
//         let profile = &mut ctx.accounts.profile;
//         profile.authority = ctx.accounts.user.key();
//         profile.name = name;
//         profile.ipfs_hash = ipfs_hash;
//         Ok(())
//     }

//     // --- 2. POST BOUNTY ---
//     pub fn post_bounty(
//         ctx: Context<PostBounty>, 
//         id: u64, 
//         price: u64, 
//         description: String,
//         arbiter: Pubkey 
//     ) -> Result<()> {
//         let bounty = &mut ctx.accounts.bounty;
//         bounty.id = id;
//         bounty.poster = ctx.accounts.poster.key();
//         bounty.mint = ctx.accounts.mint.key();
//         bounty.price = price;
//         bounty.description = description;
//         bounty.state = BountyState::Open;
//         bounty.worker = None;
//         bounty.arbiter = arbiter;
//         bounty.submission = None;
//         bounty.candidates = Vec::new();

//         // TRANSFER USDC: Poster -> Bounty Vault
//         let transfer_ctx = CpiContext::new(
//             ctx.accounts.token_program.to_account_info(),
//             Transfer {
//                 from: ctx.accounts.poster_token_account.to_account_info(),
//                 to: ctx.accounts.bounty_vault.to_account_info(),
//                 authority: ctx.accounts.poster.to_account_info(),
//             },
//         );
//         token::transfer(transfer_ctx, price)?;

//         Ok(())
//     }

//     // --- 3. APPLY ---
//     pub fn apply_for_bounty(ctx: Context<ApplyBounty>) -> Result<()> {
//         let bounty = &mut ctx.accounts.bounty;
//         require!(bounty.state == BountyState::Open, BountyError::InvalidState);
//         let applicant = ctx.accounts.applicant.key();
//         if !bounty.candidates.contains(&applicant) {
//             bounty.candidates.push(applicant);
//         }
//         Ok(())
//     }

//     // --- 4. ACCEPT CANDIDATE ---
//     pub fn accept_candidate(ctx: Context<UpdateBounty>, candidate: Pubkey) -> Result<()> {
//         let bounty = &mut ctx.accounts.bounty;
//         require!(ctx.accounts.poster.key() == bounty.poster, BountyError::Unauthorized);
//         require!(bounty.state == BountyState::Open, BountyError::InvalidState);
//         require!(bounty.candidates.contains(&candidate), BountyError::NotACandidate);

//         bounty.worker = Some(candidate);
//         bounty.state = BountyState::InProgress;
//         Ok(())
//     }

//     // --- 5. SUBMIT WORK ---
//     pub fn submit_work(ctx: Context<UpdateBounty>, link: String) -> Result<()> {
//         let bounty = &mut ctx.accounts.bounty;
//         require!(bounty.worker == Some(ctx.accounts.poster.key()), BountyError::Unauthorized);
//         require!(bounty.state == BountyState::InProgress, BountyError::InvalidState);
//         bounty.submission = Some(link);
//         bounty.state = BountyState::Review;
//         Ok(())
//     }

//     // --- 6. APPROVE & PAY ---
//     pub fn approve_work(ctx: Context<PayoutBounty>) -> Result<()> {
//         let bounty = &mut ctx.accounts.bounty;
//         require!(ctx.accounts.signer.key() == bounty.poster, BountyError::Unauthorized);
//         require!(bounty.state == BountyState::Review, BountyError::InvalidState);

//         let amount = bounty.price;

//         // ✅ FIX 1: Create a longer-lived variable for the ID bytes
//         let id_bytes = bounty.id.to_le_bytes();
        
//         let seeds = &[
//             b"bounty".as_ref(),
//             bounty.poster.as_ref(),
//             id_bytes.as_ref(), // Use the variable, not the method call
//             &[ctx.bumps.bounty],
//         ];
//         let signer = &[&seeds[..]];

//         // ✅ FIX 2: Use 'bounty.to_account_info()' directly to avoid re-borrowing from context
//         let transfer_ctx = CpiContext::new_with_signer(
//             ctx.accounts.token_program.to_account_info(),
//             Transfer {
//                 from: ctx.accounts.bounty_vault.to_account_info(),
//                 to: ctx.accounts.worker_token_account.to_account_info(),
//                 authority: bounty.to_account_info(), 
//             },
//             signer,
//         );
//         token::transfer(transfer_ctx, amount)?;

//         bounty.state = BountyState::Completed;
//         Ok(())
//     }

//     // --- 7. RAISE DISPUTE ---
//     pub fn raise_dispute(ctx: Context<UpdateBounty>) -> Result<()> {
//         let bounty = &mut ctx.accounts.bounty;
//         let is_poster = ctx.accounts.poster.key() == bounty.poster;
//         let is_worker = Some(ctx.accounts.poster.key()) == bounty.worker;
        
//         require!(is_poster || is_worker, BountyError::Unauthorized);
//         require!(bounty.state == BountyState::Review || bounty.state == BountyState::InProgress, BountyError::InvalidState);

//         bounty.state = BountyState::Dispute;
//         Ok(())
//     }

//     // --- 8. RESOLVE DISPUTE ---
//     pub fn resolve_dispute(ctx: Context<PayoutBounty>, winner_is_worker: bool) -> Result<()> {
//         let bounty = &mut ctx.accounts.bounty;
//         require!(ctx.accounts.signer.key() == bounty.arbiter, BountyError::Unauthorized);
//         require!(bounty.state == BountyState::Dispute, BountyError::InvalidState);

//         let amount = bounty.price;
        
//         // ✅ FIX 1: Same fix for ID bytes
//         let id_bytes = bounty.id.to_le_bytes();

//         let seeds = &[
//             b"bounty".as_ref(),
//             bounty.poster.as_ref(),
//             id_bytes.as_ref(),
//             &[ctx.bumps.bounty],
//         ];
//         let signer = &[&seeds[..]];

//         let destination = if winner_is_worker {
//             ctx.accounts.worker_token_account.to_account_info()
//         } else {
//             // In a real V2, you would pass the poster's token account specifically.
//             // For now, we assume the UI passed the winner's account in this slot.
//             ctx.accounts.worker_token_account.to_account_info() 
//         };

//         // ✅ FIX 2: Same fix for Authority borrowing
//         let transfer_ctx = CpiContext::new_with_signer(
//             ctx.accounts.token_program.to_account_info(),
//             Transfer {
//                 from: ctx.accounts.bounty_vault.to_account_info(),
//                 to: destination,
//                 authority: bounty.to_account_info(),
//             },
//             signer,
//         );
//         token::transfer(transfer_ctx, amount)?;

//         bounty.state = BountyState::Resolved;
//         Ok(())
//     }
// }

// // --- DATA STRUCTURES ---

// #[derive(Accounts)]
// #[instruction(name: String, ipfs_hash: String)]
// pub struct CreateProfile<'info> {
//     #[account(
//         init,
//         payer = user,
//         space = 8 + 32 + (4 + 50) + (4 + 100), 
//         seeds = [b"profile", user.key().as_ref()],
//         bump
//     )]
//     pub profile: Account<'info, UserProfile>,
//     #[account(mut)]
//     pub user: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }

// #[derive(Accounts)]
// #[instruction(id: u64)]
// pub struct PostBounty<'info> {
//     #[account(
//         init,
//         payer = poster,
//         space = 1200, 
//         seeds = [b"bounty", poster.key().as_ref(), id.to_le_bytes().as_ref()],
//         bump
//     )]
//     pub bounty: Account<'info, Bounty>,
    
//     #[account(
//         init,
//         payer = poster,
//         seeds = [b"vault", bounty.key().as_ref()],
//         bump,
//         token::mint = mint,
//         token::authority = bounty,
//     )]
//     pub bounty_vault: Account<'info, TokenAccount>,
    
//     #[account(mut)]
//     pub poster_token_account: Account<'info, TokenAccount>,
//     pub mint: Account<'info, Mint>,
    
//     #[account(mut)]
//     pub poster: Signer<'info>,
//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// pub struct PayoutBounty<'info> {
//     #[account(
//         mut,
//         seeds = [b"bounty", bounty.poster.as_ref(), bounty.id.to_le_bytes().as_ref()],
//         bump
//     )]
//     pub bounty: Account<'info, Bounty>,
    
//     #[account(
//         mut,
//         seeds = [b"vault", bounty.key().as_ref()],
//         bump,
//         token::mint = bounty.mint,
//         token::authority = bounty,
//     )]
//     pub bounty_vault: Account<'info, TokenAccount>,
    
//     #[account(mut)]
//     pub worker_token_account: Account<'info, TokenAccount>, 
    
//     #[account(mut)]
//     pub signer: Signer<'info>, 
//     pub token_program: Program<'info, Token>,
// }

// #[derive(Accounts)]
// pub struct UpdateBounty<'info> {
//     #[account(mut)]
//     pub bounty: Account<'info, Bounty>,
//     #[account(mut)]
//     pub poster: Signer<'info>, 
// }

// #[derive(Accounts)]
// pub struct ApplyBounty<'info> {
//     #[account(mut)]
//     pub bounty: Account<'info, Bounty>,
//     #[account(mut)]
//     pub applicant: Signer<'info>, 
// }

// #[account]
// pub struct Bounty {
//     pub id: u64,
//     pub poster: Pubkey,
//     pub mint: Pubkey,
//     pub price: u64,
//     pub description: String,
//     pub state: BountyState,
//     pub worker: Option<Pubkey>,
//     pub candidates: Vec<Pubkey>,
//     pub submission: Option<String>,
//     pub arbiter: Pubkey, 
// }

// #[account]
// pub struct UserProfile {
//     pub authority: Pubkey,
//     pub name: String,
//     pub ipfs_hash: String,
// }

// #[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
// pub enum BountyState { Open, InProgress, Review, Completed, Cancelled, Dispute, Resolved }

// #[error_code]
// pub enum BountyError {
//     #[msg("Unauthorized.")] Unauthorized,
//     #[msg("Invalid state.")] InvalidState,
//     #[msg("Wrong worker.")] WrongWorker,
//     #[msg("Too many candidates.")] TooManyCandidates,
//     #[msg("Not a candidate.")] NotACandidate,
// }