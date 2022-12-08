use solana_program::pubkey::Pubkey;

pub fn find_multisig_wallet_address(
    base: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"MultisigWallet".as_ref(),
            base.as_ref(),
        ],
        &multisig_demo::ID,
    ).0
}

pub fn find_multisig_transaction_address(
    multisig_wallet: &Pubkey,
    tx_nonce: u64,
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"MultisigTransaction".as_ref(),
            multisig_wallet.as_ref(),
            tx_nonce.to_le_bytes().as_ref(),
        ],
        &multisig_demo::ID,
    ).0
}
