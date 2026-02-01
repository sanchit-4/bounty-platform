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