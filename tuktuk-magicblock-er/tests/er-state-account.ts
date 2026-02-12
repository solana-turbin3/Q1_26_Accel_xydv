import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  LAMPORTS_PER_SOL,
  PublicKey,
  SendTransactionError,
} from "@solana/web3.js";
import { GetCommitmentSignature } from "@magicblock-labs/ephemeral-rollups-sdk";
import { ErStateAccount } from "../target/types/er_state_account";
import { init, taskKey, taskQueueAuthorityKey } from "@helium/tuktuk-sdk";

describe("er-state-account", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.EPHEMERAL_PROVIDER_ENDPOINT ||
        "https://devnet-as.magicblock.app/",
      {
        wsEndpoint:
          process.env.EPHEMERAL_WS_ENDPOINT || "wss://devnet.magicblock.app/",
      }
    ),
    anchor.Wallet.local()
  );
  console.log("Base Layer Connection: ", provider.connection.rpcEndpoint);
  console.log(
    "Ephemeral Rollup Connection: ",
    providerEphemeralRollup.connection.rpcEndpoint
  );
  console.log(`Current SOL Public Key: ${anchor.Wallet.local().publicKey}`);

  const Queue = new PublicKey("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh");
  const ERQueue = new PublicKey("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc");

  before(async function () {
    const balance = await provider.connection.getBalance(
      anchor.Wallet.local().publicKey
    );
    console.log("Current balance is", balance / LAMPORTS_PER_SOL, " SOL", "\n");
  });

  const program = anchor.workspace.erStateAccount as Program<ErStateAccount>;

  const taskQueue = new anchor.web3.PublicKey(
    "9SCSz59mANKxt6S8j3eRd6AAbZkYcUHuAHCcpPQa5Sr6"
  );

  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("queue_authority")],
    program.programId
  )[0];

  const taskQueueAuthority = taskQueueAuthorityKey(
    taskQueue,
    queueAuthority
  )[0];

  console.log("queueAuthority: ", queueAuthority);

  // program with ephemeral provider
  const ephemeralProgram = new anchor.Program(
    program.idl,
    providerEphemeralRollup
  ) as typeof program;

  const userAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user"), anchor.Wallet.local().publicKey.toBuffer()],
    program.programId
  )[0];

  xit("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("User Account initialized: ", tx);
  });

  it("Schedule a task!", async () => {
    let tuktukProgram = await init(provider);
    let taskId = 100;
    // Add your test here.
    const tx = await program.methods
      .schedule(taskId)
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        taskQueue: taskQueue,
        taskQueueAuthority: taskQueueAuthority,
        task: taskKey(taskQueue, taskId)[0],
        queueAuthority: queueAuthority,
        tuktukProgram: tuktukProgram.programId,
      })
      .rpc();

    console.log("Scheduled a task: ", tx);

    const userDataOld = await program.account.userAccount.fetch(userAccount);
    console.log("olds user data: ", userDataOld.data.toString());
    await sleep(10000);
    const userDataNew = await program.account.userAccount.fetch(userAccount);
    console.log("new user data: ", userDataNew.data.toString());
  });

  xit("Update State!", async () => {
    const tx = await program.methods
      .update(new anchor.BN(42))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
      })
      .rpc();
    console.log("\nUser Account State Updated: ", tx);
  });

  xit("Update state using randomness (undelegated)", async () => {
    const seed = Math.floor(Math.random() * 255);

    const tx = await program.methods
      .requestData(seed)
      .accounts({ payer: anchor.Wallet.local().publicKey, oracleQueue: Queue })
      .rpc();

    console.log("tx: ", tx);

    await sleep(5000); // takes 3-5 seconds

    const userData = await program.account.userAccount.fetch(userAccount);
    console.log("new user data: ", userData.data.toString());
  });

  xit("Delegate to Ephemeral Rollup!", async () => {
    let tx = await program.methods
      .delegate()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        validator: new PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57"),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log("\nUser Account Delegated to Ephemeral Rollup: ", tx);
  });

  xit("Update State and Commit to Base Layer!", async () => {
    let tx = await program.methods
      .updateCommit(new anchor.BN(43))
      .accountsPartial({
        user: providerEphemeralRollup.wallet.publicKey,
        userAccount: userAccount,
      })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], {
      skipPreflight: false,
    });
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection
    );

    console.log("\nUser Account State Updated: ", txHash);
  });

  xit("Update state using randomness (delegated)", async () => {
    const seed = Math.floor(Math.random() * 255);

    const tx = await ephemeralProgram.methods
      .requestData(seed)
      .accounts({
        payer: anchor.Wallet.local().publicKey,
        oracleQueue: ERQueue,
      })
      .rpc();

    console.log("tx: ", tx);

    await sleep(5000); // takes 3-5 seconds

    const userData = await program.account.userAccount.fetch(userAccount);
    console.log("new user data: ", userData.data.toString());
  });

  xit("Commit and undelegate from Ephemeral Rollup!", async () => {
    let info = await providerEphemeralRollup.connection.getAccountInfo(
      userAccount
    );

    console.log("User Account Info: ", info);

    console.log("User account", userAccount.toBase58());

    let tx = await program.methods
      .undelegate()
      .accounts({
        user: providerEphemeralRollup.wallet.publicKey,
      })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], {
      skipPreflight: false,
    });
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection
    );

    console.log("\nUser Account Undelegated: ", txHash);
  });

  xit("Update State!", async () => {
    let tx = await program.methods
      .update(new anchor.BN(45))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
      })
      .rpc();

    console.log("\nUser Account State Updated: ", tx);
  });

  xit("Close Account!", async () => {
    const tx = await program.methods
      .close()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("\nUser Account Closed: ", tx);
  });
});

const sleep = (ms: number): Promise<void> => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};
