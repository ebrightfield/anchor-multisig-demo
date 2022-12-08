use anchor_client::anchor_lang::Id;
use anchor_client::anchor_lang::InstructionData;
use anchor_client::anchor_lang::ToAccountMetas;
use anchor_client::anchor_lang::AccountDeserialize;
use anchor_client::anchor_lang::prelude::System;
use anyhow::Result;
use solana_client::client_error::ClientErrorKind;
use solana_sdk::instruction::Instruction;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::{RpcError, RpcResponseErrorData};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use multisig_demo::state::{MultisigTransaction, MultisigWallet};
use crate::pda::{find_multisig_transaction_address, find_multisig_wallet_address};

pub fn fetch_transaction(addr: &Pubkey, client: &RpcClient) -> Result<MultisigTransaction> {
    let act_data = client.get_account_data(&addr)?;
    let tx = MultisigTransaction::try_deserialize(&mut act_data.as_slice())?;
    Ok(tx)
}

/// Create a new multisig wallet
pub fn new_multisig_rpc(
    threshold: u16,
    members: Vec<Pubkey>,
    client: &RpcClient,
    payer: &dyn Signer,
    base: Option<&dyn Signer>,
) -> Result<Signature> {
    let ix = multisig_demo::instruction::NewMultisig {
        threshold,
        members,
    };
    let maybe_key = Keypair::new();
    let base = base.unwrap_or(&maybe_key);
    let multisig_wallet = find_multisig_wallet_address(&base.pubkey());
    let acts = multisig_demo::accounts::NewMultisig {
        base: base.pubkey(),
        payer: payer.pubkey(),
        multisig_wallet,
        system_program: System::id(),
    };
    let ix = Instruction {
        data: ix.data(),
        accounts: acts.to_account_metas(None),
        program_id: multisig_demo::ID,
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer, base],
        client.get_latest_blockhash()?
    );
    Ok(client.send_transaction(&tx)?)
}

/// Association between a signer and a multisig wallet.
/// This allows for abstracting away many implementation details
/// for constructing transaction instructions and RPC calls.
pub struct MultisigMember {
    member: Box<dyn Signer>,
    multisig_address: Pubkey,
    multisig_data: MultisigWallet,
    client: RpcClient,
}

impl MultisigMember {
    /// This constructor pulls the multisig wallet metadata from on-chain.
    pub fn try_new(
        member: Box<dyn Signer>,
        multisig_address: Pubkey,
        client: RpcClient,
    ) -> Result<Self> {
        let act_data = client.get_account_data(&multisig_address)?;
        let multisig_data = MultisigWallet::try_deserialize(&mut act_data.as_slice())?;
        Ok(Self {
            member,
            multisig_address,
            multisig_data,
            client,
        })
    }

    /// Pull data from on-chain, mostly just in case the `tx_nonce` has
    /// incremented.
    pub fn refresh_wallet(&mut self) -> Result<()> {
        let act_data = self.client.get_account_data(&self.multisig_address)?;
        self.multisig_data = MultisigWallet::try_deserialize(&mut act_data.as_slice())?;
        Ok(())
    }

    /// Returns the pubkey of the next proposal produced on this multisig wallet,
    /// which updates every [NewTransaction] instruction with the increment of
    /// the `member_set_seqno`.
    pub fn next_transaction_pubkey(&self) -> Pubkey {
        find_multisig_transaction_address(
            &self.multisig_address, self.multisig_data.tx_nonce)
    }

    /// RPC call to propose a change of the approval threshold.
    pub fn propose_change_threshold(&self, threshold: u16) -> Result<Signature> {
        let ix = multisig_demo::instruction::ChangeThreshold {
            threshold
        };
        let acts = multisig_demo::accounts::Administration {
            multisig_wallet: self.multisig_address.clone()
        };
        let ix = Instruction {
            data: ix.data(),
            accounts: acts.to_account_metas(None),
            program_id: multisig_demo::ID,
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)?)
    }

    /// RPC call to propose a change of the member set.
    pub fn propose_change_members(&self, members: Vec<Pubkey>) -> Result<Signature> {
        let ix = multisig_demo::instruction::ChangeMembers {
            members
        };
        let acts = multisig_demo::accounts::Administration {
            multisig_wallet: self.multisig_address.clone()
        };
        let ix = Instruction {
            data: ix.data(),
            accounts: acts.to_account_metas(None),
            program_id: multisig_demo::ID,
        };
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)?)
    }

    /// Create the instruction to create a new [MultisigTransaction].
    pub fn new_transaction_ix(&mut self, instructions: Vec<Instruction>) -> Instruction {
        let ix = multisig_demo::instruction::NewTransaction {
            instructions: instructions.into_iter().map(|ix| ix.into()).collect(),
        };
        let transaction = self.next_transaction_pubkey();
        let acts = multisig_demo::accounts::NewTransaction {
            proposer: self.member.pubkey(),
            multisig_wallet: self.multisig_address.clone(),
            transaction,
            system_program: System::id(),
        };
        Instruction {
            data: ix.data(),
            accounts: acts.to_account_metas(None),
            program_id: multisig_demo::ID,
        }
    }

    /// RPC call to create a new transaction proposal under this object's
    /// multisig wallet.
    pub fn new_transaction_rpc(&mut self, instructions: Vec<Instruction>) -> Result<Signature> {
        let ix = self.new_transaction_ix(instructions);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)?)
    }

    /// Create the instruction to approve a [MultisigTransaction].
    pub fn approve_ix(&self, transaction: Pubkey) -> Instruction {
        let ix = multisig_demo::instruction::Approve;
        let acts = multisig_demo::accounts::Approval {
            member: self.member.pubkey(),
            transaction,
            multisig_wallet: self.multisig_address.clone(),
        };
        Instruction {
            data: ix.data(),
            accounts: acts.to_account_metas(None),
            program_id: multisig_demo::ID,
        }
    }

    /// RPC call to approve a [MultisigTransaction].
    pub fn approve_rpc(&self, transaction: Pubkey) -> Result<Signature> {
        let ix = self.approve_ix(transaction);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)
            .map_err(|e| maybe_print_preflight_simulation_logs(e))
            ?)
    }

    /// Create the instruction to cancel approval of a [MultisigTransaction].
    pub fn unapprove_ix(&self, transaction: Pubkey) -> Instruction {
        let ix = multisig_demo::instruction::Unapprove;
        let acts = multisig_demo::accounts::Approval {
            member: self.member.pubkey(),
            transaction,
            multisig_wallet: self.multisig_address.clone(),
        };
        Instruction {
            data: ix.data(),
            accounts: acts.to_account_metas(None),
            program_id: multisig_demo::ID,
        }
    }

    /// RPC call to cancel approval of a [MultisigTransaction].
    pub fn unapprove_rpc(&self, transaction: Pubkey) -> Result<Signature> {
        let ix = self.unapprove_ix(transaction);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)
            .map_err(|e| maybe_print_preflight_simulation_logs(e))
            ?)
    }

    /// Create the instruction to execute a [MultisigTransaction].
    pub fn execute_ix(&self, transaction: Pubkey, data: MultisigTransaction) -> Instruction {
        let ix = multisig_demo::instruction::Execute;
        let acts = multisig_demo::accounts::Approval {
            member: self.member.pubkey(),
            transaction,
            multisig_wallet: self.multisig_address.clone(),
        };
        let mut act_metas = acts.to_account_metas(None);
        data.instructions
            .into_iter()
            .for_each(|ix| {
                ix.keys
                    .into_iter()
                    .for_each(|key| {
                        if key.pubkey != self.multisig_address {
                            act_metas.push(key.into());
                        }
                    });
                act_metas.push(AccountMeta {
                    pubkey: ix.program_id,
                    is_signer: false,
                    is_writable: false
                });
            });
        act_metas.push(AccountMeta {
            pubkey: self.multisig_address,
            is_signer: false,
            is_writable: false
        });
        Instruction {
            data: ix.data(),
            accounts: act_metas,
            program_id: multisig_demo::ID,
        }
    }

    /// RPC call to execute a [MultisigTransaction].
    pub fn execute_rpc(&self, transaction: Pubkey) -> Result<Signature> {
        let data = fetch_transaction(&transaction, &self.client)?;
        let ix = self.execute_ix(transaction, data);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)
            .map_err(|e| maybe_print_preflight_simulation_logs(e))
            ?)
    }

    /// RPC call with a compound transaction for convenience,
    /// issuing both the [CreateTransaction] and [Approve] instructions.
    pub fn create_and_approve_tx(&mut self, instructions: Vec<Instruction>) -> Result<Signature> {
        let ix = self.new_transaction_ix(instructions);
        let transaction = self.next_transaction_pubkey();
        let ix2 = self.approve_ix(transaction);
        let tx = Transaction::new_signed_with_payer(
            &[ix, ix2],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)
            .map_err(|e| maybe_print_preflight_simulation_logs(e))
            ?)
    }

    /// RPC call with a compound transaction for convenience,
    /// issuing both the [Approve] and [Execute] instructions.
    pub fn approve_and_execute_tx(&self, transaction: Pubkey) -> Result<Signature> {
        let ix = self.approve_ix(transaction);
        let data = fetch_transaction(&transaction, &self.client)?;
        let ix2 = self.execute_ix(transaction, data);
        let tx = Transaction::new_signed_with_payer(
            &[ix, ix2],
            Some(&self.member.pubkey()),
            &[self.member.as_ref()],
            self.client.get_latest_blockhash()?
        );
        Ok(self.client.send_transaction(&tx)
            .map_err(|e| maybe_print_preflight_simulation_logs(e))
            ?)
    }
}

/// Prints the transaction logs for failed preflight simulations.
/// Otherwise just prints the error.
/// Returns the error back out for any further desired processing.
pub fn maybe_print_preflight_simulation_logs(
    err: solana_client::client_error::ClientError
) -> solana_client::client_error::ClientError {
    if let ClientErrorKind::RpcError(err) = &err.kind {
        if let RpcError::RpcResponseError { data, .. } = err {
            // print the transaction logs for a failed pre-flight simulation
            if let RpcResponseErrorData::SendTransactionPreflightFailure(
                result
            ) = data {
                if let Some(logs) = &result.logs {
                    logs.iter().for_each(|e| println!("{}", e))
                }
            }
        }
    }
    err
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_instructions() {
        let signer = Box::new(Keypair::new());
        let base = Pubkey::new_unique();
        let (multisig, bump) = Pubkey::find_program_address(
            &[
                b"MultisigWallet".as_ref(),
                base.as_ref(),
            ],
            &multisig_demo::ID,
        );
        let client = RpcClient::new_mock("succeeds");
        let data = MultisigWallet {
            base,
            members: vec![signer.pubkey()],
            threshold: 1,
            tx_nonce: 0,
            member_set_seqno: 0,
            bump,
        };
        let mut member = MultisigMember {
            member: signer,
            multisig_address: multisig,
            multisig_data: data,
            client
        };
        let transaction = find_multisig_transaction_address(
            &multisig, 0,
        );
        let _ix = member.new_transaction_ix(vec![]);
        let _ix = member.approve_ix(transaction.clone());
        let _ix = member.unapprove_ix(transaction.clone());
        let _ix = member.execute_ix(transaction.clone(),
            MultisigTransaction::default(),
        );
    }
}
