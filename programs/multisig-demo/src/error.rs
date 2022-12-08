use anchor_lang::error_code;



#[error_code]
pub enum MultisigError {
    #[msg("Threshold must be <= the number of multisig wallet members")]
    InvalidThreshold,
    #[msg("Members of a multisig must be unique addresses")]
    DuplicateMembers,
    #[msg("Not enough members with the given threshold")]
    TooFewMembers,
    #[msg("Not a current member of the multisig wallet")]
    NotAMember,
    #[msg("The multisig wallet does not match the transaction's member_set_seqno")]
    InvalidMemberSetSeqno,
    #[msg("The transaction does not belong to the provided multisig")]
    InvalidMultisigReference,
    #[msg("Signer already approved this transaction")]
    AlreadyApproved,
    #[msg("Signer already is marked as unapproved for this transaction.")]
    AlreadyUnapproved,
    #[msg("Transaction requires more approvals before it can be executed")]
    NotEnoughApprovals,
    #[msg("Transaction already executed")]
    AlreadyExecuted,
}
