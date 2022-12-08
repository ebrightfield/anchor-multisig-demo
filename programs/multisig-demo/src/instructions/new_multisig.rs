use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;
use vipers::invariant;
use anchor_lang::prelude::{Account, Program, Signer, System};
use crate::state::MultisigWallet;
use crate::error::MultisigError;

#[derive(Accounts)]
#[instruction(threshold: u16, members: Vec<Pubkey>)]
pub struct NewMultisig<'info> {
    /// Entropy to ensure a unique address.
    /// Improves on a design flaw in Project Serum's multisig implementation.
    base: Signer<'info>,
    /// Funds the creation of the wallet. Does not need to be contained in [members].
    #[account(mut)]
    payer: Signer<'info>,
    /// The PDA which stores relevant state, and which also signs approved transactions
    /// during their execution.
    #[account(
        init,
        seeds = [
            b"MultisigWallet".as_ref(),
            &base.key().as_ref()
        ],
        bump,
        payer = payer,
        space = MultisigWallet::space(members.len()),
    )]
    multisig_wallet: Account<'info, MultisigWallet>,
    system_program: Program<'info, System>,
}

impl<'info> NewMultisig<'info> {
    pub fn validate(&self, threshold: &u16, members: &Vec<Pubkey>) -> Result<()> {
        // Cannot have a threshold of zero
        invariant!(*threshold > 0, MultisigError::InvalidThreshold);
        // Cannot have a threshold higher than the number of members
        invariant!(*threshold as usize <= members.len(), MultisigError::InvalidThreshold);
        // Members must be unique
        let mut deduped = members.clone();
        deduped.dedup();
        invariant!(members.len() == deduped.len(), MultisigError::DuplicateMembers);
        Ok(())
    }

    pub fn handle(&mut self, threshold: u16, members: Vec<Pubkey>, bump: u8) -> Result<()> {
        let msig = &mut self.multisig_wallet;
        msig.member_set_seqno = 0;
        msig.tx_nonce = 0;
        msig.members = members.clone();
        msig.base = self.base.key();
        msig.threshold = threshold;
        msig.bump = bump;
        Ok(())
    }
}
