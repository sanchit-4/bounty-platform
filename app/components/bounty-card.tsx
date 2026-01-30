// 'use client';

// import { useWallet } from '@solana/wallet-adapter-react';
// import { PublicKey } from '@solana/web3.js';
// import { useGigBoardProgram } from '../hooks/useGigBoardProgram';

// export function BountyCard({ account, publicKey }: { account: any, publicKey: PublicKey }) {
//   const { program } = useGigBoardProgram();
//   const wallet = useWallet();
  
//   // Helper to check ownership
//   const isPoster = wallet.publicKey?.toString() === account.poster.toString();
//   const isWorker = wallet.publicKey?.toString() === account.worker?.toString();
  
//   const state = Object.keys(account.state)[0].toLowerCase(); // 'open', 'submitted', etc.

//   // ACTION: Poster Approves Work
//   const handleApprove = async () => {
//     // You need to derive the WORKER PROFILE PDA here to pass it to the instruction
//     const [workerProfilePda] = PublicKey.findProgramAddressSync(
//         [Buffer.from("user"), account.worker.toBuffer()],
//         program.programId
//     );

//     await program.methods.approveWork()
//       .accounts({
//         bounty: publicKey,
//         poster: wallet.publicKey,
//         worker: account.worker,
//         workerProfile: workerProfilePda,
//       })
//       .rpc();
//   };

//   // ACTION: Worker Submits
//   const handleSubmitWork = async () => {
//     const link = prompt("Enter submission URL:");
//     if(!link) return;
//     await program.methods.submitWork(link).accounts({
//        bounty: publicKey,
//        worker: wallet.publicKey
//     }).rpc();
//   };

//   // ACTION: Apply/Claim (Simplified for MVP)
//   const handleApply = async () => {
//     // For this MVP, let's say clicking "Apply" auto-assigns you (simplification)
//     // Real app: You apply -> Poster sees list -> Poster assigns
//     await program.methods.assignWorker(wallet.publicKey).accounts({
//         bounty: publicKey,
//         poster: wallet.publicKey // Note: In this snippet, only Poster can assign. 
//         // You would need a separate UI for Poster to paste a worker address.
//     }).rpc();
//   };

//   return (
//     <div className="border border-gray-700 bg-gray-800 p-4 rounded-lg mb-4">
//       <div className="flex justify-between items-center mb-2">
//         <h3 className="text-lg font-bold text-white">{account.description}</h3>
//         <span className={`px-2 py-1 rounded text-xs uppercase ${
//             state === 'resolved' ? 'bg-green-500' : 'bg-blue-500'
//         }`}>
//           {state}
//         </span>
//       </div>
      
//       <p className="text-gray-400 text-sm mb-4">Price: {account.price.toString() / 1e9} SOL</p>

//       <div className="flex gap-2">
//         {/* LOGIC: If Open and I am Poster -> Show "Waiting for applicants" */}
//         {state === 'open' && isPoster && (
//              <p className="text-yellow-500 text-sm">Waiting for workers...</p>
//         )}

//         {/* LOGIC: If Assigned and I am Worker -> Show Submit */}
//         {state === 'assigned' && isWorker && (
//              <button onClick={handleSubmitWork} className="bg-blue-600 px-4 py-2 rounded text-white">
//                 Submit Work
//              </button>
//         )}

//         {/* LOGIC: If Submitted and I am Poster -> Show Approve */}
//         {state === 'submitted' && isPoster && (
//              <button onClick={handleApprove} className="bg-green-600 px-4 py-2 rounded text-white">
//                 Approve & Pay
//              </button>
//         )}
//       </div>
//     </div>
//   );
// }


'use client';

import { useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { useGigBoardProgram } from '../hooks/useGigBoardProgram';
import { toast } from 'react-hot-toast';

const lamportsToSol = (lamports: any) => (parseInt(lamports.toString()) / 1000000000).toFixed(2);

const getStatus = (stateObj: any) => {
    if (stateObj.open) return "Open";
    if (stateObj.inProgress) return "In Progress";
    if (stateObj.review) return "Review";
    if (stateObj.completed) return "Completed";
    return "Cancelled";
};

export function BountyCard({ account, publicKey }: { account: any, publicKey: PublicKey }) {
    const { publicKey: userKey } = useWallet();
    const { program } = useGigBoardProgram();
    const [loading, setLoading] = useState(false);

    const status = getStatus(account.state);
    const isPoster = userKey && account.poster.toString() === userKey.toString();
    const isWorker = userKey && account.worker && account.worker.toString() === userKey.toString();
    
    // Check if current user has already applied
    const hasApplied = userKey && account.candidates.find((c: PublicKey) => c.toString() === userKey.toString());

    // --- ACTIONS ---

    // 1. Volunteer (Apply)
    const handleApply = async () => {
        if (!program || !userKey) return;
        try {
            setLoading(true);
            await program.methods
                .applyForBounty()
                .accounts({
                    bounty: publicKey,
                    applicant: userKey,
                })
                .rpc();
            toast.success("Applied! Wait for the owner to select you.");
            window.location.reload();
        } catch (error) {
            console.error(error);
            toast.error("Failed to apply");
        } finally { setLoading(false); }
    };

    // 2. Accept Candidate (Owner only)
    const handleAcceptCandidate = async (candidateKey: PublicKey) => {
        if (!program || !isPoster) return;
        try {
            setLoading(true);
            await program.methods
                .acceptCandidate(candidateKey)
                .accounts({
                    bounty: publicKey,
                    poster: userKey,
                    workerAccount: candidateKey, 
                })
                .rpc();
            toast.success("Worker Selected!");
            window.location.reload();
        } catch (error: any) {
            console.error(error);
            
            // ✅ FIX: Check if the error is actually a success message
            if (error.message && error.message.includes("already been processed")) {
                toast.success("Worker Selected!");
                window.location.reload();
                return;
            }

            // Only show error if it's NOT the "already processed" one
            toast.error("Failed to select worker");
        } finally { setLoading(false); }
    };

    // 3. Submit Work (Worker only)
    const handleSubmitWork = async () => {
        const link = prompt("Enter proof of work link:");
        if (!link || !program || !userKey) return; // Ensure userKey exists
        
        try {
            setLoading(true);
            await program.methods
                .submitWork(link)
                .accounts({
                    bounty: publicKey,
                    poster: userKey,
                    // ✅ FIX: Pass the worker's key here to satisfy the struct
                    workerAccount: userKey, 
                })
                .rpc();
            toast.success("Work Submitted!");
            window.location.reload();
        } catch (error: any) {
            console.error(error);
            // Handle the "already processed" false positive here too
            if (error.message && error.message.includes("already been processed")) {
                toast.success("Work Submitted!");
                window.location.reload();
                return;
            }
            toast.error("Failed to submit");
        } finally { setLoading(false); }
    };

    // 4. Approve (Owner only)
    const handleApprove = async () => {
        if (!program) return;
        try {
            setLoading(true);
            await program.methods
                .approveWork()
                .accounts({
                    bounty: publicKey,
                    // @ts-expect-error
                    poster: userKey,
                    workerAccount: account.worker, // Ensure this exists
                })
                .rpc();
            toast.success("Paid out!");
            window.location.reload();
        } catch (error: any) {
            console.error(error);
            // ✅ FIX: Ignore "already processed" error
            if (error.message && error.message.includes("already been processed")) {
                toast.success("Paid out!");
                window.location.reload();
                return;
            }
            toast.error("Failed to approve");
        } finally { setLoading(false); }
    };

    return (
        <div className="bg-slate-800 p-6 rounded-xl border border-slate-700 shadow-lg relative">
            <div className={`absolute top-0 right-0 px-4 py-1 text-xs font-bold uppercase ${
                status === 'Open' ? 'bg-green-500 text-black' : 
                status === 'In Progress' ? 'bg-blue-500 text-white' : 
                status === 'Review' ? 'bg-yellow-500 text-black' : 'bg-purple-600 text-white'
            }`}>
                {status}
            </div>

            <h3 className="text-xl font-bold text-white mb-2">{account.description}</h3>
            <p className="text-2xl font-bold text-purple-400 mb-4">{lamportsToSol(account.price)} SOL</p>
            {/* Visible if state is Review or Completed, and a link exists */}
            {(status === 'Review' || status === 'Completed') && account.submission && (
                <div className="mb-4 bg-slate-900 p-3 rounded border border-slate-700">
                    <p className="text-xs text-gray-400 uppercase mb-1">Worker's Submission:</p>
                    <a 
                        href={account.submission.startsWith('http') ? account.submission : `https://${account.submission}`} 
                        target="_blank" 
                        rel="noreferrer"
                        className="text-blue-400 underline break-all hover:text-blue-300"
                    >
                        {account.submission}
                    </a>
                </div>
            )}
            {/* --- SECTION: CANDIDATES (Only visible to Poster when Open) --- */}
            {status === 'Open' && isPoster && (
                <div className="mb-4 bg-slate-900 p-3 rounded">
                    <h4 className="text-xs text-gray-400 mb-2 uppercase">Applicants ({account.candidates.length})</h4>
                    {account.candidates.length === 0 && <p className="text-sm text-gray-500 italic">No volunteers yet...</p>}
                    
                    {account.candidates.map((cand: PublicKey) => (
                        <div key={cand.toString()} className="flex justify-between items-center mb-2">
                            <span className="text-xs text-white font-mono">{cand.toString().slice(0, 8)}...</span>
                            <button 
                                onClick={() => handleAcceptCandidate(cand)}
                                disabled={loading}
                                className="bg-green-600 hover:bg-green-500 text-white text-xs px-2 py-1 rounded"
                            >
                                Select
                            </button>
                        </div>
                    ))}
                </div>
            )}

            {/* --- ACTION BUTTONS --- */}
            <div className="flex gap-2">
                {/* 1. Worker: Volunteer */}
                {status === 'Open' && !isPoster && !hasApplied && (
                    <button 
                        onClick={handleApply} disabled={loading}
                        className="w-full bg-slate-600 hover:bg-slate-500 text-white py-2 rounded"
                    >
                        Volunteer for this Gig
                    </button>
                )}
                {status === 'Open' && hasApplied && (
                    <div className="w-full text-center text-yellow-500 text-sm font-bold border border-yellow-500/30 p-2 rounded">
                        Applied! Pending selection...
                    </div>
                )}

                {/* 2. Worker: Submit */}
                {status === 'In Progress' && isWorker && (
                    <button 
                        onClick={handleSubmitWork} disabled={loading}
                        className="w-full bg-indigo-600 hover:bg-indigo-500 text-white py-2 rounded"
                    >
                        Submit Work
                    </button>
                )}

                {/* 3. Owner: Approve */}
                {status === 'Review' && isPoster && (
                    <button 
                        onClick={handleApprove} disabled={loading}
                        className="w-full bg-green-600 hover:bg-green-500 text-white py-2 rounded"
                    >
                        Approve & Pay Worker
                    </button>
                )}
                
                {status === 'Review' && isWorker && (
                    <div className="w-full text-center text-yellow-500 italic">Waiting for approval...</div>
                )}
            </div>
        </div>
    );
}