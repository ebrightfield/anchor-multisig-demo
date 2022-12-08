use anchor_lang::prelude::*;
use vipers::{assert_keys_eq, invariant};
use crate::state::{MultisigTransaction, MultisigWallet};
use crate::error::MultisigError;
use crate::gen_multisig_wallet_seeds;

#[derive(Accounts)]
pub struct Approval<'info> {
    #[account(mut)]
    member: Signer<'info>,
    #[account(mut)]
    multisig_wallet: Account<'info, MultisigWallet>,
    #[account(mut)]
    transaction: Account<'info, MultisigTransaction>,
}

impl<'info> Approval<'info> {
    pub fn validate(&self) -> Result<()> {
        let msig = &self.multisig_wallet;
        let tx = &self.transaction;
        // There should be a chain of references from Transaction -> Multisig -> Signer.
        assert_keys_eq!(tx.multisig_wallet, msig.key(),
            MultisigError::InvalidMultisigReference
        );
        invariant!(msig.members.contains(&self.member.key()),
            MultisigError::NotAMember,
        );
        // The transaction should not be executed yet.
        invariant!(tx.executed_at.is_none(), MultisigError::AlreadyExecuted);
        invariant!(tx.executor.is_none(), MultisigError::AlreadyExecuted);
        Ok(())
    }

    pub fn handle_approve(&mut self) -> Result<()> {
        // First find the index where we need to mark a [true] in the transaction approvals.
        let member = self.member.key();
        let member_idx = self.multisig_wallet.members.binary_search(&member)
            .map_err(|_| MultisigError::NotAMember)?;

        // If the member has already approved, throw an error.
        // This allows preflight simulations to catch unnecessary approval transactions.
        let tx = &mut self.transaction;
        invariant!(tx.approved[member_idx].is_none(),
            MultisigError::AlreadyApproved);
        tx.approved[member_idx] = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }

    pub fn handle_unapprove(&mut self) -> Result<()> {
        // First find the index where we need to mark a [false] in the transaction approvals.
        let member = self.member.key();
        let member_idx = self.multisig_wallet.members.binary_search(&member)
            .map_err(|_| MultisigError::NotAMember)?;

        // If the member is already marked as unapproved, throw an error.
        // This allows preflight simulations to catch unnecessary unapproval transactions.
        let tx = &mut self.transaction;
        invariant!(tx.approved[member_idx].is_some(),
            MultisigError::AlreadyUnapproved);
        tx.approved[member_idx] = None;
        Ok(())
    }
    pub fn handle_execute(&mut self, remaining_accounts: &[AccountInfo]) -> Result<()> {
        let msig = &self.multisig_wallet;
        // Approval check
        let num_approvals: usize = self.transaction.approved
            .iter()
            .filter(|&approved| approved.is_some())
            .count();
        invariant!(num_approvals >= msig.threshold as usize, MultisigError::NotEnoughApprovals);

        let seeds = gen_multisig_wallet_seeds!(self.multisig_wallet);

        // Perform transaction as CPI
        for ix in self.transaction.instructions.iter() {
            solana_program::program::invoke_signed(
                &ix.clone().into(),
                remaining_accounts,
                &[&seeds[..]],
            )?;
        }
        // We need to reload the wallet here to persist any possible mutations
        // that may have occurred.
        self.multisig_wallet.reload()?;

        let tx = &mut self.transaction;
        tx.executor = Some(self.member.key());
        tx.executed_at = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }
}
