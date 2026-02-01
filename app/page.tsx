'use client';

import dynamic from 'next/dynamic'; // <--- 1. Add this import
import { CreateBountyForm } from './components/create-bounty-form';
import { BountyList } from './components/bounty-list';

// 2. Create a "Client Only" version of the button
const WalletMultiButtonDynamic = dynamic(
  async () => (await import('@solana/wallet-adapter-react-ui')).WalletMultiButton,
  { ssr: false }
);

export default function Page() {
  return (
    <div className="min-h-screen bg-slate-900 text-white p-4 md:p-8">
      <nav className="flex justify-between items-center mb-12 border-b border-slate-700 pb-4">
        <h1 className="text-3xl font-extrabold bg-gradient-to-r from-purple-400 to-pink-600 text-transparent bg-clip-text">
          Solana Gig-Board
        </h1>
        
        {/* 3. Use the Dynamic Button instead of the standard one */}
        <WalletMultiButtonDynamic className="!bg-purple-700 hover:!bg-purple-600" />
      
      </nav>

      <main className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-1">
          <div className="sticky top-8">
            <CreateBountyForm />
            <div className="mt-8 p-4 bg-slate-800/50 rounded-xl text-sm text-gray-400">
              <h3 className="font-bold text-white mb-2">How it works</h3>
              <ul className="list-disc pl-4 space-y-2">
                <li>Post a bounty (Funds are locked in Escrow).</li>
                <li>Workers submit proof of work.</li>
                <li>You approve, and the contract pays them instantly.</li>
              </ul>
            </div>
          </div>
        </div>

        <div className="lg:col-span-2">
          <BountyList />
        </div>
      </main>
    </div>
  );
}