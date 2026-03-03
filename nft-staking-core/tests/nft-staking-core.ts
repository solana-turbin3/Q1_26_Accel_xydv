import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStakingCore } from "../target/types/nft_staking_core";
import { Keypair, SendTransactionError, SystemProgram } from "@solana/web3.js";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

const MILLISECONDS_PER_DAY = 86400000;
const POINTS_PER_STAKED_NFT_PER_DAY = 10_000_000;
const FREEZE_PERIOD_IN_DAYS = 7;
const TIME_TRAVEL_IN_DAYS = 8;

describe("nft-staking-core", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.nftStakingCore as Program<NftStakingCore>;

  // Generate a keypair for the collection
  const collectionKeypair = anchor.web3.Keypair.generate();

  // Find the update authority for the collection (PDA)
  const updateAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("update_authority"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];

  // Generate a keypair for the nft asset
  const nftKeypair = anchor.web3.Keypair.generate();

  // Find the config account (PDA)
  const config = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), collectionKeypair.publicKey.toBuffer()],
    program.programId
  )[0];

  // Find the rewards mint account (PDA)
  const rewardsMint = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("rewards"), config.toBuffer()],
    program.programId
  )[0];

  const oracleAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("oracle")],
    program.programId
  )[0];

  console.log("oracleAccount ", oracleAccount);

  const rewardsAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reward_vault"), oracleAccount.toBuffer()],
    program.programId
  )[0];

  /**
   * Helper function to advance time with surfnet_timeTravel RPC method
   * @param params - Time travel params (absoluteEpoch, absoluteSlot, or absoluteTimestamp)
   */
  async function advanceTime(params: {
    absoluteEpoch?: number;
    absoluteSlot?: number;
    absoluteTimestamp?: number;
  }): Promise<void> {
    const rpcResponse = await fetch(provider.connection.rpcEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "surfnet_timeTravel",
        params: [params],
      }),
    });

    const result = (await rpcResponse.json()) as { error?: any; result?: any };
    if (result.error) {
      throw new Error(`Time travel failed: ${JSON.stringify(result.error)}`);
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  it("Create a collection", async () => {
    const collectionName = "Test Collection";
    const collectionUri = "https://example.com/collection";
    const tx = await program.methods
      .createCollection(collectionName, collectionUri)
      .accountsPartial({
        payer: provider.wallet.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([collectionKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("Collection address", collectionKeypair.publicKey.toBase58());
  });

  it("Mint an NFT", async () => {
    const nftName = "Test NFT";
    const nftUri = "https://example.com/nft";
    const tx = await program.methods
      .mintNft(nftName, nftUri)
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nftKeypair])
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log("NFT address", nftKeypair.publicKey.toBase58());
  });

  it("Initialize stake config", async () => {
    const tx = await program.methods
      .initializeConfig(POINTS_PER_STAKED_NFT_PER_DAY, FREEZE_PERIOD_IN_DAYS)
      .accountsPartial({
        admin: provider.wallet.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ skipPreflight: true });
    console.log("\nYour transaction signature", tx);
    console.log("Config address", config.toBase58());
    console.log("Points per staked NFT per day", POINTS_PER_STAKED_NFT_PER_DAY);
    console.log("Freeze period in days", FREEZE_PERIOD_IN_DAYS);
    console.log("Rewards mint address", rewardsMint.toBase58());
  });

  it("Transfer an NFT (self) - success", async () => {
    const oracle = await program.account.oracle.all();
    console.log("\nUpdated Oracle", oracle[0].account.validation);

    const tx = await program.methods
      .transfer()
      .accountsPartial({
        user: provider.wallet.publicKey,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        newOwner: provider.publicKey,
        oracle: oracleAccount,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([])
      .rpc({ skipPreflight: true });
  });

  it("Time travel to not allowed time", async () => {
    // Advance time in milliseconds
    const currentTimestamp = Date.now();
    await advanceTime({
      absoluteTimestamp: currentTimestamp + 5 * 60 * 60 * 1000,
    });
  });

  it("Update oracle", async () => {
    const tx = await program.methods
      .updateOracle()
      .accountsPartial({
        payer: provider.wallet.publicKey,
        oracle: oracleAccount,
        rewardVault: rewardsAccount,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const oracle = await program.account.oracle.all();
    console.log("\nUpdated Oracle", oracle[0].account.validation);
  });

  // this one would fail
  it("Transfer an NFT (self) - failed", async () => {
    try {
      const tx = await program.methods
        .transfer()
        .accountsPartial({
          user: provider.wallet.publicKey,
          nft: nftKeypair.publicKey,
          collection: collectionKeypair.publicKey,
          newOwner: provider.publicKey,
          oracle: oracleAccount,
          systemProgram: SystemProgram.programId,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([])
        .rpc({ skipPreflight: true });
    } catch (e) {
      // console.log(e);
    }
  });

  it("Stake an NFT", async () => {
    const tx = await program.methods
      .stake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("\nYour transaction signature", tx);
  });

  it("Time travel 1 day", async () => {
    // Advance time in milliseconds
    const currentTimestamp = Date.now();
    await advanceTime({
      absoluteTimestamp: currentTimestamp + 2 * MILLISECONDS_PER_DAY,
    });
    console.log("\nTime traveled in days", 1);
  });

  it("claim rewards without unstaking", async () => {
    // Get the user rewards ATA account
    const userRewardsAta = getAssociatedTokenAddressSync(
      rewardsMint,
      provider.wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const tx = await program.methods
      .claimRewards()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        userRewardsAta,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log(
      "User rewards balance",
      (await provider.connection.getTokenAccountBalance(userRewardsAta)).value
        .uiAmount
    );
  });

  it("Time travel 8 days", async () => {
    // Advance time in milliseconds
    const currentTimestamp = Date.now();
    await advanceTime({
      absoluteTimestamp:
        currentTimestamp + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY,
    });
    console.log("\nTime traveled in days", TIME_TRAVEL_IN_DAYS);
  });

  it("Burn staked NFT", async () => {
    // Get the user rewards ATA account
    const userRewardsAta = getAssociatedTokenAddressSync(
      rewardsMint,
      provider.wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const tx = await program.methods
      .burnStakedNft()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        userRewardsAta,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("\nYour transaction signature", tx);
    console.log(
      "User rewards balance",
      (await provider.connection.getTokenAccountBalance(userRewardsAta)).value
        .uiAmount
    );
  });

  xit("Unstake an NFT", async () => {
    // Get the user rewards ATA account
    const userRewardsAta = getAssociatedTokenAddressSync(
      rewardsMint,
      provider.wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const tx = await program.methods
      .unstake()
      .accountsPartial({
        user: provider.wallet.publicKey,
        updateAuthority,
        config,
        rewardsMint,
        userRewardsAta,
        nft: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("\nYour transaction signature", tx);
    console.log(
      "User rewards balance",
      (await provider.connection.getTokenAccountBalance(userRewardsAta)).value
        .uiAmount
    );
  });
});
