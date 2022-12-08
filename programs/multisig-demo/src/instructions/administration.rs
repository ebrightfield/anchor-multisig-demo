use anchor_lang::prelude::*;
use vipers::{invariant, unwrap_int};
use crate::error::MultisigError;
use crate::state::MultisigWallet;

/// Used for actions that modify the multisig account itself,
/// and which therefore require the multisig account to sign
/// as an authorization measure. This means instructions that use
/// this [Accounts] must be embedded in a [MultisigTransaction]
/// to successfully execute.
#[derive(Accounts)]
pub struct Administration<'info> {
    #[account(mut, signer)]
    multisig_wallet: Account<'info, MultisigWallet>,
}

impl<'info> Administration<'info> {
    pub fn handle_change_threshold(&mut self, threshold: u16) -> Result<()> {
        invariant!(threshold as usize <= self.multisig_wallet.members.len(),
            MultisigError::InvalidThreshold
        );
        self.multisig_wallet.threshold = threshold;
        Ok(())
    }

    pub fn handle_change_members(self: &mut Self, members: Vec<Pubkey>) -> Result<()> {
        invariant!(members.len() >= self.multisig_wallet.threshold as usize,
            MultisigError::TooFewMembers,
        );
        self.multisig_wallet.members = members;
        self.multisig_wallet.member_set_seqno = unwrap_int!(
            self.multisig_wallet.member_set_seqno.checked_add(1));
        Ok(())
    }
}