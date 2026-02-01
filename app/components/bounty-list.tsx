'use client';

import { useEffect, useState, useMemo } from 'react';
import { useGigBoardProgram } from '../hooks/useGigBoardProgram';
import { BountyCard } from './bounty-card'; 
import { PublicKey } from '@solana/web3.js';

export function BountyList() {
  const { program } = useGigBoardProgram();
  const [bounties, setBounties] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  // 1. Stabilize the dependency. 
  // We convert the Program ID to a string. Strings are easy for React to compare.
  // If 'program' is null, this string is undefined. If it exists, it's a constant string.
  const programIdString = useMemo(() => 
    program?.programId?.toString() || "", 
  [program]);
  useEffect(() => {
    // If no program yet, don't do anything
    if (!program) return;

    const fetchBounties = async () => {
      try {
        setLoading(true);
        // @ts-expect-error
        const allBounties = await program.account.bounty.all();
        
        const sorted = allBounties.sort((a: any, b: any) => {
          return b.account.id.toNumber() - a.account.id.toNumber();
        });

        setBounties(sorted);
      } catch (error) {
        console.error("Error fetching bounties:", error);
      } finally {
        setLoading(false);
      }
    };

    fetchBounties();

    // 2. CRITICAL: Only re-run if 'programIdString' changes. 
    // We intentionally OMIT 'program' from this array to stop the infinite loop.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [programIdString]); 

  if (loading && bounties.length === 0) {
    return <div className="text-white animate-pulse">Loading Gig Board...</div>;
  }

  if (!loading && bounties.length === 0) {
    return (
      <div className="text-center text-gray-400 py-10">
        No gigs posted yet. Be the first!
      </div>
    );
  }

  return (
    <div className="grid gap-4">
        <div className="flex justify-between items-center mb-4">
            <h2 className="text-2xl font-bold text-white">Active Bounties</h2>
            {/* Manual Refresh Button - Safer than auto-refreshing */}
            <button 
                onClick={() => window.location.reload()} 
                className="text-sm text-purple-400 hover:text-purple-300 underline"
            >
                Refresh Board
            </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {bounties.map((bounty) => (
                <BountyCard 
                    key={bounty.publicKey.toString()} 
                    account={bounty.account} 
                    publicKey={bounty.publicKey} 
                />
            ))}
        </div>
    </div>
  );
}