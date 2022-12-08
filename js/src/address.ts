import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";

export function findMultisigWalletAddress(
  base: anchor.web3.PublicKey,
  program: anchor.web3.PublicKey,
): anchor.web3.PublicKey {
  let [addr, _] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("MultisigWallet"),
      base.toBuffer(),
    ],
    program,
  );
  return addr;
}
export function findMultisigTransactionAddress(
  multisigWallet: anchor.web3.PublicKey,
  txNonce: anchor.BN,
  program: anchor.web3.PublicKey,
): anchor.web3.PublicKey {
  let [addr, _] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("MultisigTransaction"),
      multisigWallet.toBuffer(),
      txNonce.toBuffer('le', 8),
    ],
    program,
  );
  return addr;
}
