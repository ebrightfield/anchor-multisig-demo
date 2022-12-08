use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

/// PDA that represents an M of N multisig signer, and all relevant metadata.
#[account]
#[derive(Default, Debug, PartialEq)]
pub struct MultisigWallet {
    /// Base used to derive.
    pub base: Pubkey,
    /// Members able to create and approve a [MultisigTransaction] owned by this
    /// multisig.
    pub members: Vec<Pubkey>,
    /// At least this many members needed to approve [MultisigTransaction]
    /// before it can be executed.
    pub threshold: u16,
    /// To ensure uniqueness of [MultisigTransaction] PDA address.
    pub tx_nonce: u64,
    /// This is used to make sure that a transaction created with owner set X
    /// cannot be approved by a new owner set Y. This handles the edge cases
    /// where a user approves a transaction and then leaves multisig membership.
    pub member_set_seqno: u32,
    pub bump: u8,
}

impl MultisigWallet {
    pub fn space(num_members: usize) -> usize {
        8 + // Anchor Account Discriminator
        32 + // base: Pubkey,
        4 + // members: Vec length (u32)
        32 *  num_members + // members: 32 bytes per Pubkey
        2 + // threshold: u16
        8 + // tx_nonce: u64
        4 + // owner_set_seqno: u64
        1 // bump
    }
}

#[macro_export]
macro_rules! gen_multisig_wallet_seeds {
    ($multisig_wallet:expr) => {
        &[
             b"MultisigWallet".as_ref(),
             $multisig_wallet.base.as_ref(),
             &[$multisig_wallet.bump],
        ]
    };
}

/// PDA that represents a proposed transaction, and contains all relevant metadata.
#[account]
#[derive(Debug, Default, PartialEq)]
pub struct MultisigTransaction {
    /// The instruction set of the transaction to be executed.
    pub instructions: Vec<Instruction>,
    /// The [MultisigWallet] account this transaction belongs to.
    pub multisig_wallet: Pubkey,
    /// Keeps track of which accounts approved the transaction, and when
    /// their last approval occurred.
    /// `approved[i]` is Some iff `[MultisigWallet.members[i]]` signed the transaction.
    pub approved: Vec<Option<i64>>,
    /// Saved to ensure that no approvals occur on a different set of memberships.
    pub member_set_seqno: u32,
    /// Unix timestamp at time of the [MultisigTransaction] account's creation.
    pub created_at: i64,
    /// The account that executed the [Transaction].
    pub proposer: Pubkey,
    /// The account that executed the [Transaction].
    pub executor: Option<Pubkey>,
    /// If/when the transaction was executed.
    pub executed_at: Option<i64>,
}

impl MultisigTransaction {
    pub fn space(instructions: Vec<Instruction>, num_members: usize) -> usize {
            8 + // Anchor Account Discriminator
            4 + // instructions: Vec length: u32
            (instructions
                .iter()
                .map(|ix| ix.space())
                .sum::<usize>()) + // instructions: Vec<Instruction>,
            32 + // multisig_wallet: Pubkey,
            4 + // approved: Vec length: u32
            (1 + 8) * num_members + // approved: Vec<Option<i64>>,
            4 + // member_set_seqno: u64
            8 + // created_at: i64
            32 + // proposer: Pubkey,
            (1 + 32) + // executor: Option<Pubkey>,
            (1 + 8) // executed_at: Option<Pubkey>,
    }
}

/// Anchor (de-)serializable version of [solana_program::instruction::Instruction].
/// Note that all member variable names are deliberately chosen to match the JS
/// library, so that normal Solana instruction serialization should "just work" client-side.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq)]
pub struct Instruction {
    /// The program that this instruction invokes
    pub program_id: Pubkey,
    /// [AccountMeta] listing which accounts are to be read/written-to during
    /// execution.
    pub keys: Vec<AccountMeta>,
    /// Serialized instruction data
    pub data: Vec<u8>,
}

impl Instruction {
    pub fn space(&self) -> usize {
        32 + // program_id: Pubkey,
        4 + // keys: Vec length (u32)
        AccountMeta::LEN * self.keys.len() +
        4 + // data: Vec length (u32)
        self.data.len()
    }
}

impl Into<solana_program::instruction::Instruction> for Instruction {
    fn into(self) -> solana_program::instruction::Instruction {
        solana_program::instruction::Instruction {
            program_id: self.program_id.clone(),
            accounts: self.keys.clone().into_iter().map(Into::into).collect(),
            data: self.data.clone(),
        }
    }
}

impl Into<Instruction> for solana_program::instruction::Instruction {
    fn into(self) -> Instruction {
        Instruction {
            program_id: self.program_id.clone(),
            keys: self.accounts.clone().into_iter().map(Into::into).collect(),
            data: self.data.clone(),
        }
    }
}



/// Anchor (de-)serializable version of [solana_program::instruction::AccountMeta]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, PartialEq, Copy, Clone)]
pub struct AccountMeta {
    /// An account that will be read or written to during transaction execution.
    pub pubkey: Pubkey,
    /// True if an Instruction requires a Transaction signature matching `pubkey`.
    pub is_signer: bool,
    /// True if the `pubkey` can be loaded as a read-write account.
    pub is_writable: bool,
}

impl AccountMeta {
    pub const LEN: usize =
        32 + // pubkey: Pubkey
        1 + // is_signer: bool
        1; // is_writable: bool
}

impl Into<solana_program::instruction::AccountMeta> for AccountMeta {
    fn into(self) -> solana_program::instruction::AccountMeta {
        solana_program::instruction::AccountMeta {
            pubkey: self.pubkey.clone(),
            is_signer: self.is_signer.clone(),
            is_writable: self.is_writable.clone(),
        }
    }
}

impl Into<AccountMeta> for solana_program::instruction::AccountMeta {
    fn into(self) -> AccountMeta {
        AccountMeta {
            pubkey: self.pubkey.clone(),
            is_signer: self.is_signer.clone(),
            is_writable: self.is_writable.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use anchor_lang::solana_program::system_instruction::transfer;
    use super::*;

    const DISCRIMINATOR_BYTE_SIZE: usize = 8;

    #[test]
    fn multisig_wallet_length() {
        assert_eq!(
            MultisigWallet::space(0),
            DISCRIMINATOR_BYTE_SIZE + MultisigWallet::default().try_to_vec().unwrap().len(),
        );
        assert_eq!(
            MultisigWallet::space(1),
            DISCRIMINATOR_BYTE_SIZE + MultisigWallet::default().try_to_vec().unwrap().len() + std::mem::size_of::<Pubkey>(),
        );
    }

    #[test]
    fn multisig_transaction_length() {
        // Note -- If executor and executed_at were left [None],
        // the unit tests here would fail, it seems Option types are serializing
        // to one byte when they're None, but 1 + n bytes when they're Some(T), where
        // n is the length of a serialized T.
        assert_eq!(
            MultisigTransaction::space(vec![], 0),
            DISCRIMINATOR_BYTE_SIZE +
                MultisigTransaction {
                    executor: Some(Default::default()),
                    executed_at: Some(Default::default()),
                    ..Default::default()
                }.try_to_vec().unwrap().len(),
        );
        // Note: Still 40 bytes extra here too
        let mut tx = MultisigTransaction::default();
        let ix = transfer(
            &Default::default(), &Default::default(), 0);
        tx.instructions = vec![ix.clone().into()];
        tx.executor = Some(Default::default());
        tx.executed_at = Some(Default::default());
        assert_eq!(
            MultisigTransaction::space(vec![ix.clone().into()], 0),
            DISCRIMINATOR_BYTE_SIZE +
                tx.try_to_vec().unwrap().len(),
        );
    }
}
