use anchor_lang::prelude::*;
use vipers::invariant;
use crate::state::{Instruction, MultisigTransaction, MultisigWallet};
use crate::error::MultisigError;

#[derive(Accounts)]
#[instruction(instructions: Vec<Instruction>)]
pub struct NewTransaction<'info> {
    /// Must be a member of the given [multisig_wallet].
    #[account(mut)]
    proposer: Signer<'info>,
    /// The wallet whose address is intended to sign the transaction
    /// upon execution. Only members of this wallet can sign for tx approval.
    #[account(mut)]
    multisig_wallet: Account<'info, MultisigWallet>,
    #[account(
        init,
        seeds=[
            b"MultisigTransaction".as_ref(),
            &multisig_wallet.key().as_ref(),
            &multisig_wallet.tx_nonce.to_le_bytes().as_ref(),
        ],
        bump,
        payer=proposer,
        space=MultisigTransaction::space(instructions, multisig_wallet.members.len()),
    )]
    transaction: Account<'info, MultisigTransaction>,
    system_program: Program<'info, System>,
}

impl<'info> NewTransaction<'info> {
    pub fn validate(&self) -> Result<()> {
        invariant!(self.multisig_wallet.members.contains(&self.proposer.key()),
            MultisigError::NotAMember,
        );
        Ok(())
    }

    pub fn handle(&mut self, instructions: Vec<Instruction>) -> Result<()> {
        let tx = &mut self.transaction;
        // Transaction Content
        tx.instructions = instructions;
        // Authorization Controls
        tx.multisig_wallet = self.multisig_wallet.key();
        tx.approved = (0..self.multisig_wallet.members.len())
            .map(|_| None)
            .collect();
        tx.member_set_seqno = self.multisig_wallet.member_set_seqno;
        // Transaction History
        tx.created_at = Clock::get()?.unix_timestamp;
        tx.proposer = self.proposer.key();
        tx.executed_at = None;
        tx.executor = None;

        // Increment the multisig nonce, so that the next transaction has a unique public key.
        let msig = &mut self.multisig_wallet;
        msig.tx_nonce += 1;
        Ok(())
    }
}
