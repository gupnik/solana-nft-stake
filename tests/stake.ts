import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Stake } from '../target/types/stake';
import {
  AccountLayout,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MintLayout,
  Token,
  TOKEN_PROGRAM_ID,
  u64,
} from "@solana/spl-token";

const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new anchor.web3.PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');

describe('stake', () => {

  const provider = anchor.Provider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Stake as Program<Stake>;

  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();

  let jamboToken;

  it('Is initialized!', async () => {

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        payer.publicKey,
        100 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );

    const [stakeAccount, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake"), payer.publicKey.toBuffer()],
      program.programId,
    );
    console.log('Stake Account', stakeAccount.toBase58(), bump);

    // Add your test here.
    const tx = await program.rpc.initialize(
      bump,
      {
        accounts: {
          stakeAccount,
          authority: payer.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          // rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [payer]
      }
    );
    console.log("Your transaction signature", tx);
  });

  it('can stake!', async () => {
    jamboToken = await Token.createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );
    console.log('Jambo Token', jamboToken.publicKey.toBase58());

    const jamboMintAccount = await jamboToken.getOrCreateAssociatedAccountInfo(
      payer.publicKey
    );
    console.log('Jambo Account', jamboMintAccount.address.toBase58());

    await jamboToken.mintTo(jamboMintAccount.address, mintAuthority.publicKey, [mintAuthority], 1);
    console.log('Minted');

    const [stakeJamboAccount, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake"), jamboToken.publicKey.toBuffer()],
      program.programId,
    );
    console.log('Stake Account', stakeJamboAccount.toBase58(), bump);

    const prevAccount = await program.account.stakeJamboAccount.fetchNullable(stakeJamboAccount);
    console.log('Prev Account', prevAccount);

    const [jamboMintPool, jamboMintPoolBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [stakeJamboAccount.toBuffer(), Buffer.from("jamboMintPool")],
      program.programId
    );
    console.log('Jambo Mint Pool', jamboMintPool.toBase58());

    const tx = await program.rpc.stakeJambo(
      bump,
      {
        accounts: {
          stakeJamboAccount,
          authority: payer.publicKey,
          jamboMint: jamboToken.publicKey,
          jamboMintSrc: jamboMintAccount.address,
          jamboMintPool,
          tokenProgram: TOKEN_PROGRAM_ID,
          // associatedTokenProgram: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [payer]
      }
    );
    console.log("Your transaction 2 signature", tx);

    const account = await program.account.stakeJamboAccount.fetch(stakeJamboAccount);
    console.log(account.jamboMint.toBase58(), account.bump);
  });

  it('can unstake!', async () => {
    console.log('Jambo Token', jamboToken.publicKey.toBase58());

    const jamboMintAccount = await jamboToken.getOrCreateAssociatedAccountInfo(
      payer.publicKey
    );
    console.log('Jambo Account', jamboMintAccount.address.toBase58());

    const [stakeJamboAccount, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake"), jamboToken.publicKey.toBuffer()],
      program.programId,
    );
    console.log('Stake Account', stakeJamboAccount.toBase58(), bump);

    const [jamboMintPool, jamboMintPoolBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [stakeJamboAccount.toBuffer(), Buffer.from("jamboMintPool")],
      program.programId
    );
    console.log('Jambo Mint Pool', jamboMintPool.toBase58());

    const tx = await program.rpc.unstakeJambo(
      {
        accounts: {
          stakeJamboAccount,
          authority: payer.publicKey,
          jamboMint: jamboToken.publicKey,
          jamboMintDest: jamboMintAccount.address,
          jamboMintPool,
          tokenProgram: TOKEN_PROGRAM_ID,
          // associatedTokenProgram: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [payer]
      }
    );
    console.log("Your transaction 3 signature", tx);

    const account = await program.account.stakeJamboAccount.fetchNullable(stakeJamboAccount);
    console.log(account.jamboMint.toBase58(), account.bump);
  });

  it('can stake again!', async () => {
    console.log('Jambo Token', jamboToken.publicKey.toBase58());

    const jamboMintAccount = await jamboToken.getOrCreateAssociatedAccountInfo(
      payer.publicKey
    );
    console.log('Jambo Account', jamboMintAccount.address.toBase58());

    const [stakeJamboAccount, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake"), jamboToken.publicKey.toBuffer()],
      program.programId,
    );
    console.log('Stake Account', stakeJamboAccount.toBase58(), bump);

    const [jamboMintPool, jamboMintPoolBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [stakeJamboAccount.toBuffer(), Buffer.from("jamboMintPool")],
      program.programId
    );
    console.log('Jambo Mint Pool', jamboMintPool.toBase58());

    const tx = await program.rpc.restakeJambo(
      {
        accounts: {
          stakeJamboAccount,
          authority: payer.publicKey,
          jamboMint: jamboToken.publicKey,
          jamboMintSrc: jamboMintAccount.address,
          jamboMintPool,
          tokenProgram: TOKEN_PROGRAM_ID,
          // associatedTokenProgram: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [payer]
      }
    );
    console.log("Your transaction 4 signature", tx);

    const account = await program.account.stakeJamboAccount.fetchNullable(stakeJamboAccount);
    console.log(account.jamboMint.toBase58(), account.bump);
  });
});
