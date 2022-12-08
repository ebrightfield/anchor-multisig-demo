import * as anchor from "@project-serum/anchor";
import {Instruction, Program} from "@project-serum/anchor";
import { MultisigDemo } from "./multisig_demo";
import {AccountMeta, Commitment, Transaction, TransactionInstruction, TransactionSignature} from "@solana/web3.js";
import {findMultisigTransactionAddress, findMultisigWalletAddress} from "./address";

/// Replicated the type here because pulling it out of the IDL is beyond my TS skills.
export type MultisigWallet = {
  base: anchor.web3.PublicKey;
  members: anchor.web3.PublicKey[];
  threshold: number;
  txNonce: anchor.BN;
  memberSetSeqno: number;
  bump: number;
};

/// A combination of signer + multisig wallet.
/// Hides implementation details associated with interacting with the program.
/// Broadly speaking, the interface is exposed such that users can easily
/// create transaction instructions, or directly make RPC calls.
export class MultisigMember {
  /// Assumed to be a member of the multisig wallet
  private readonly signer: anchor.web3.Signer;
  /// Multisig Wallet Public Key
  public readonly walletAddress: anchor.web3.PublicKey;
  /// Multisig Wallet Account Data
  public wallet: MultisigWallet;
  /// Anchor `Program` functionality gets wrapped by this class.
  private readonly program: Program<MultisigDemo>;

  constructor(
    signer: anchor.web3.Signer,
    walletAddress: anchor.web3.PublicKey,
    wallet: MultisigWallet,
    program: Program<MultisigDemo>,
  ) {
    this.signer = signer;
    this.walletAddress = walletAddress;
    this.wallet = wallet;
    this.program = program;
  }

  /// Use this factory function if you don't care to separately fetch
  /// the `MultisigWallet` metadata.
  static async newFromAddress(
    signer: anchor.web3.Signer,
    walletAddress: anchor.web3.PublicKey,
    program: Program<MultisigDemo>,
    commitment?: Commitment,
  ): Promise<MultisigMember> {
    const wallet = await program.account.multisigWallet.fetch(walletAddress, commitment);
    return new MultisigMember(
      signer,
      walletAddress,
      wallet,
      program,
    );
  }

  async refreshWallet(commitment?: Commitment) {
    this.wallet = await this.program.account.multisigWallet.fetch(this.walletAddress,
      commitment
      );
  }

  nextTransactionAddress() {
    return findMultisigTransactionAddress(
      this.walletAddress,
      this.wallet.txNonce,
      this.program.programId,
    );
  }

  /// A common execution routine used all the RPC methods.
  async sendTx(
    instructions: TransactionInstruction[],
    signers: anchor.web3.Signer[],
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {
    const conn = anchor.getProvider().connection;
    const tx = new Transaction({
        feePayer: this.signer.publicKey,
        ...(await conn.getLatestBlockhash(confirmOptions?.commitment)),
    });
    tx.add(...instructions);
    tx.sign(...signers);

    return await conn.sendTransaction(tx, signers, confirmOptions);
  }

  /// Optional commitment is passed to a refresh of the multisig wallet metadata.
  async newTransactionIx(
    instructions: TransactionInstruction[],
    commitment?: Commitment,
  ): Promise<TransactionInstruction> {

    // Refresh our data, so that we're sure we're using an up-to-date nonce.
    await this.refreshWallet(commitment);
    const multisigTransaction = this.nextTransactionAddress();
    return await this.program.methods.newTransaction(
      instructions
    )
      .accounts({
        proposer: this.signer.publicKey,
        multisigWallet: this.walletAddress,
        transaction: multisigTransaction,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.signer])
      .instruction();
  }

  async approveIx(
    transaction: anchor.web3.PublicKey,
    ): Promise<TransactionInstruction> {
    return await this.program.methods.approve()
      .accounts({
        member: this.signer.publicKey,
        multisigWallet: this.walletAddress,
        transaction,
      })
      .signers([this.signer])
      .instruction();
  }

  async unapproveIx(
    transaction: anchor.web3.PublicKey,
  ): Promise<TransactionInstruction> {
    return await this.program.methods.unapprove()
      .accounts({
        member: this.signer.publicKey,
        multisigWallet: this.walletAddress,
        transaction,
      })
      .signers([this.signer])
      .instruction();
  }

  async executeIx(
    transaction: anchor.web3.PublicKey,
    commitment?: Commitment,
  ): Promise<TransactionInstruction> {
    const txData = await this.program.account.multisigTransaction.fetch(
      transaction, commitment,
    );
    let acts: AccountMeta[] = [];
    (txData.instructions as TransactionInstruction[]).map(
      (ix) => {
        acts.push({
          pubkey: ix.programId,
          isSigner: false,
          isWritable: false,
        });
        ix.keys.map((key) => {
          if (!key.pubkey.equals(this.walletAddress)) {
            acts = acts.concat([key]);
          }
        });
      }
    )
    return await this.program.methods.execute()
      .accounts({
        member: this.signer.publicKey,
        multisigWallet: this.walletAddress,
        transaction,
      })
      .remainingAccounts([...acts,
        {
         pubkey: this.walletAddress,
         isSigner: false,
         isWritable: true,
        }])
      .signers([this.signer])
      .instruction();
  }

  async newTransactionRpc(
    instructions: TransactionInstruction[],
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {

    const ix = await this.newTransactionIx(instructions, confirmOptions?.commitment);
    return await this.sendTx(
      [ix],
      [this.signer],
      confirmOptions,
    );
  }

  async approveRpc(
    transaction: anchor.web3.PublicKey,
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {

    const ix = await this.approveIx(transaction);
    return await this.sendTx(
      [ix],
      [this.signer],
      confirmOptions,
    );
  }

  async unapproveRpc(
    transaction: anchor.web3.PublicKey,
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {

    const ix = await this.unapproveIx(transaction);
    return await this.sendTx(
      [ix],
      [this.signer],
      confirmOptions,
    );
  }

  async executeRpc(
    transaction: anchor.web3.PublicKey,
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {

    const ix = await this.executeIx(transaction, confirmOptions?.commitment);
    return await this.sendTx(
      [ix],
      [this.signer],
      confirmOptions,
    );
  }

  async newTransactionAndApproveRpc(
    instructions: TransactionInstruction[],
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {
    // Refresh our data, so that we're sure we're using an up-to-date nonce.
    await this.refreshWallet(confirmOptions?.commitment);
    const multisigTransaction = this.nextTransactionAddress();
    const newTransactionIx = await this.program.methods.newTransaction(
      instructions
    )
      .accounts({
        proposer: this.signer.publicKey,
        multisigWallet: this.walletAddress,
        transaction: multisigTransaction,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.signer])
      .instruction();
    const approveIx = await this.approveIx(multisigTransaction);
    return await this.sendTx(
      [newTransactionIx, approveIx], [this.signer], confirmOptions);
  }

  async approveAndExecuteRpc(
    transaction: anchor.web3.PublicKey,
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {
    const approveIx = await this.approveIx(transaction);
    const executeIx = await this.executeIx(transaction);
    return await this.sendTx(
      [approveIx, executeIx], [this.signer], confirmOptions);
  }

  async proposeChangeThreshold(
    threshold: number,
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {
    let ix = await this.program.methods.changeThreshold(threshold)
      .accounts({
        multisigWallet: this.walletAddress,
      })
      .instruction();
    const multisigTransaction = this.nextTransactionAddress();
    const newTransactionIx = await this.program.methods.newTransaction(
      [ix]
    )
      .accounts({
        proposer: this.signer.publicKey,
        multisigWallet: this.walletAddress,
        transaction: multisigTransaction,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.signer])
      .instruction();
    const approveIx = await this.approveIx(multisigTransaction);
    return await this.sendTx(
      [newTransactionIx], [this.signer], confirmOptions);
  }

  async proposeChangeMembers(
    members: anchor.web3.PublicKey[],
    confirmOptions?: anchor.web3.ConfirmOptions,
  ): Promise<TransactionSignature> {
    let ix = await this.program.methods.changeMembers(members)
      .accounts({
        multisigWallet: this.walletAddress,
      })
      .instruction();
    const multisigTransaction = this.nextTransactionAddress();
    const newTransactionIx = await this.program.methods.newTransaction(
      [ix]
    )
      .accounts({
        proposer: this.signer.publicKey,
        multisigWallet: this.walletAddress,
        transaction: multisigTransaction,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.signer])
      .instruction();
    const approveIx = await this.approveIx(multisigTransaction);
    return await this.sendTx(
      [newTransactionIx], [this.signer], confirmOptions);
  }


}

export async function newMultisigRpc(
  base: anchor.web3.Signer,
  payer: anchor.web3.Signer,
  threshold: number,
  members: anchor.web3.PublicKey[],
  program: Program<MultisigDemo>,
  confirmOptions?: anchor.web3.ConfirmOptions,
): Promise<TransactionSignature> {
  const multisigWallet = findMultisigWalletAddress(
    base.publicKey,
    program.programId,
  );
  return await program.methods.newMultisig(
    threshold,
    members,
  )
    .accounts({
      base: base.publicKey,
      payer: payer.publicKey,
      multisigWallet,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([
      payer,
      base,
    ])
    .rpc(confirmOptions);
}
