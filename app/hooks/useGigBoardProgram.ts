import { useAnchorWallet, useConnection } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { AnchorProvider, Program, Idl, setProvider } from '@coral-xyz/anchor';
import idl from '@/app/utils/gig_board.json'; 
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

const PROGRAM_ID = new PublicKey("8iefNKLGA5E1YCHW1thUr4QXRk7c2qVvLKngU76Cq6gc"); 

export function useGigBoardProgram() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  
  const provider = new AnchorProvider(
    connection,
    wallet as any,
    AnchorProvider.defaultOptions()
  );

  if (wallet) {
    setProvider(provider);
  }

  const program = new Program(idl as Idl, provider);

  return {
    program,
    provider,
    connection,
    wallet,
    tokenProgramId: TOKEN_PROGRAM_ID,
  };
}