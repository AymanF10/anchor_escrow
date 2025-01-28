import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorEscrow } from "../target/types/anchor_escrow";
import { BN } from "bn.js";
import { createAssociatedTokenAccountIdempotentInstruction, createInitializeMint2Instruction, createMintToInstruction, getAssociatedTokenAddressSync, getMinimumBalanceForRentExemptMint, MINT_SIZE, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { LAMPORTS_PER_SOL, sendAndConfirmTransaction, Transaction, PublicKey, SystemProgram } from "@solana/web3.js";
import { token } from "@coral-xyz/anchor/dist/cjs/utils"
import { randomBytes } from "crypto";

describe("anchor_escrow", () => {

  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const program = anchor.workspace.AnchorEscrow as Program<AnchorEscrow>;
  const maker = anchor.web3.Keypair.generate();
  const mintA = anchor.web3.Keypair.generate();
  const mintB = anchor.web3.Keypair.generate();
  const taker = anchor.web3.Keypair.generate();
  const seed = new BN(randomBytes(8));
  const tokenProgram = TOKEN_PROGRAM_ID;
  const makerAtaA = getAssociatedTokenAddressSync(mintA.publicKey, maker.publicKey, false, tokenProgram)
  const takerAtaB = getAssociatedTokenAddressSync(mintB.publicKey, taker.publicKey, false, tokenProgram);
  const [escrow] = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)], 
    program.programId
  )
  const vault = getAssociatedTokenAddressSync(mintA.publicKey, escrow, true, tokenProgram)
  
  it("airdrop", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(program.provider.connection);
    let tx = new anchor.web3.Transaction();
    tx.instructions = [
   
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: maker.publicKey,
        lamports: 0.2 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: taker.publicKey,
        lamports: 0.2 * LAMPORTS_PER_SOL,
      }),
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mintA.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: tokenProgram,
      }),
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mintB.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: tokenProgram,
      }),

    createInitializeMint2Instruction(mintA.publicKey, 6, maker.publicKey, null, tokenProgram),
    createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, makerAtaA, maker.publicKey, mintA.publicKey, tokenProgram  ),
    createMintToInstruction(mintA.publicKey, makerAtaA,maker.publicKey, 1e9, undefined, tokenProgram ),
    
    createInitializeMint2Instruction(mintB.publicKey, 6, taker.publicKey, null, tokenProgram),
    createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, takerAtaB, taker.publicKey, mintB.publicKey, tokenProgram ),                    
    createMintToInstruction(mintB.publicKey, takerAtaB ,taker.publicKey, 1e9, undefined, tokenProgram ), 
    ];
    
    
    console.log({
      maker: maker.publicKey.toString(), 
      taker: taker.publicKey.toString(), 
      mintA: mintA.publicKey.toString(), 
      mintB: mintB.publicKey.toString(),
      makerAtaA: makerAtaA.toString(),
      takerAtaB: takerAtaB.toString()
    });
    await provider.sendAndConfirm(tx, [maker, taker, mintA, mintB]);
    const connection = program.provider.connection;

    const mintAInfo = await connection.getAccountInfo(mintA.publicKey);
    console.log("MintA Account Info:", mintAInfo);

    const mintBInfo = await connection.getAccountInfo(mintB.publicKey);
    console.log("MintB Account Info:", mintBInfo);

    if (!mintAInfo || !mintBInfo) {
    throw new Error("Mint accounts are not initialized.");
    }
    const makerAtaABalance = await connection.getTokenAccountBalance(makerAtaA);
  console.log("Maker ATA A Balance:", makerAtaABalance);

  
  if (makerAtaABalance.value.amount !== "1000000000") {
    throw new Error("Incorrect token balance in maker's ATA for mintA.");
  }
  
  });
  
  it("lets make an Escrow!", async () => {
   
    const accounts = { 
      maker: maker.publicKey,  
      mintA: mintA.publicKey,
      mintB: mintB.publicKey,
      makerAtaA,
      escrow,
      vault,  
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, 
      tokenProgram,
      systemProgram: SystemProgram.programId,
    };

    console.log("Maker ATA A:", makerAtaA.toString());
    console.log("Taker ATA B:", takerAtaB.toString());
    console.log("Escrow:", escrow.toString());
    console.log("Vault:", vault.toString());

    const tx = await program.methods.make(
      seed,
      
      new BN(1),
      new BN(1),
    )
   
    .accountsPartial({...accounts})
    .signers([maker])
    
    .rpc();
    console.log("Your transaction signature", tx);
  });

});