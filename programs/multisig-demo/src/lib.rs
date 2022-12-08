pub mod state;
mod error;
mod instructions;

use anchor_lang::prelude::*;
use state::Instruction;

use instructions::*;

declare_id!("H1wPJB59dvpLrMmdcJ6dxQSMmXsm6TSRxkNUtC4CFDs1");

#[program]
pub mod multisig_demo {
    use crate::instructions::administration::Administration;
    use super::*;

    /// Initialize a new [MultisigWallet].
    #[access_control(ctx.accounts.validate(&threshold, &members))]
    pub fn new_multisig(
        ctx: Context<NewMultisig>,
        threshold: u16,
        members: Vec<Pubkey>,
        ) -> Result<()> {
        ctx.accounts.handle(
            threshold,
            members,
            *ctx.bumps.get("multisig_wallet").unwrap()
        )
    }

    /// Initialize a new [MultisigTransaction].
    #[access_control(ctx.accounts.validate())]
    pub fn new_transaction(
        ctx: Context<NewTransaction>,
        instructions: Vec<Instruction>,
    ) -> Result<()> {
        ctx.accounts.handle(instructions)
    }

    /// Approve a [MultisigTransaction] for execution.
    #[access_control(ctx.accounts.validate())]
    pub fn approve(
        ctx: Context<Approval>,
    ) -> Result<()> {
        ctx.accounts.handle_approve()
    }

    /// Cancel approval for a [MultisigTransaction].
    #[access_control(ctx.accounts.validate())]
    pub fn unapprove(
        ctx: Context<Approval>,
    ) -> Result<()> {
        ctx.accounts.handle_unapprove()
    }

    /// Execute a [MultisigTransaction], iff it has enough approvals and hasn't
    /// yet been executed.
    #[access_control(ctx.accounts.validate())]
    pub fn execute(
        ctx: Context<Approval>,
    ) -> Result<()> {
        ctx.accounts.handle_execute(ctx.remaining_accounts)
    }

    pub fn change_threshold(
        ctx: Context<Administration>,
        threshold: u16,
    ) -> Result<()> {
        ctx.accounts.handle_change_threshold(threshold)
    }

    pub fn change_members(
        ctx: Context<Administration>,
        members: Vec<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.handle_change_members(members)
    }
}
