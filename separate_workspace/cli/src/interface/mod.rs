use anyhow::Result;
use clap::{ArgMatches, Parser, IntoApp};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use anchor_client::anchor_lang::AccountDeserialize;
use multisig_demo::state::MultisigTransaction;
use multisig_demo_sdk::{find_multisig_wallet_address, MultisigMember, new_multisig_rpc};
use crate::config::{UrlArg, KeypairArg, pubkey_or_signer_path, pubkey_arg};


/// Multisig CLI
///
/// Provides an interface for direct interaction with the multisig program
/// also contained in this repo.
#[derive(Debug, Parser)]
pub struct Opts {
    #[clap(flatten)]
    pub url: UrlArg,
    #[clap(flatten)]
    pub keypair: KeypairArg,
    #[clap(subcommand)]
    pub command: Command,
}


#[derive(Debug, Parser)]
pub enum Command {
    /// Create a new multisig wallet
    NewMultisig {
        /// This flag adds the configured `-k/--keypair` signer
        #[clap(long)]
        include_signer: bool,
        /// Minimum required number of signers to execute a transaction
        /// signed by the wallet. Must be greater than zero,
        /// no greater than the total number of members.
        #[clap(long)]
        threshold: u16,
        /// List of members. Must be unique. Can be either a base-58 pubkey string,
        /// or any path compatible with the `-k/--keypair` flag.
        members: Vec<String>,
    },
    /// Demonstrate the create transaction functionality with a simple
    /// memo transaction.
    ProposeMemo {
        /// The target multisig wallet on which to propose a new transaction.
        #[clap(parse(try_from_str=pubkey_arg))]
        multisig_wallet: Pubkey,
        /// The memo message to propose (the multisig address will sign a memo
        /// instruction with this memo string).
        memo: String,
    },
    /// Approve a transaction
    Approve {
        /// Target transaction to approve.
        #[clap(parse(try_from_str=pubkey_arg))]
        transaction: Pubkey,
    },
    /// Cancel approval of a transaction
    Unapprove {
        /// Target transaction to unapprove.
        #[clap(parse(try_from_str=pubkey_arg))]
        transaction: Pubkey,
    },
    /// Execute a transaction. Requires that the threshold of approvals is reached.
    Execute {
        /// Target transaction to execute.
        #[clap(parse(try_from_str=pubkey_arg))]
        transaction: Pubkey,
    },
    /// Propose a new threshold on the given multisig.
    ProposeNewThreshold {
        /// The target multisig wallet on which to propose a new threshold.
        #[clap(parse(try_from_str=pubkey_arg))]
        multisig_wallet: Pubkey,
        /// Minimum required number of signers to execute a transaction
        /// signed by the wallet. Must be greater than zero,
        /// no greater than the total number of members.
        threshold: u16,
    },
    /// Propose a new set of members on the given multisig.
    ProposeNewMembers {
        /// The target multisig wallet on which to propose a new threshold.
        #[clap(parse(try_from_str=pubkey_arg))]
        multisig_wallet: Pubkey,
        /// List of members. Must be unique. Can be either a base-58 pubkey string,
        /// or any path compatible with the `-k/--keypair` flag.
        members: Vec<String>,
    },
}

pub fn entry(
    opts: &Opts,
    signer: Box<dyn Signer>,
    client: RpcClient,
) -> Result<()> {
    match &opts.command {
        Command::NewMultisig {
            include_signer,
            threshold,
            members,
        } => {
            let app = Opts::into_app();
            let matches = app.get_matches();
            new_multisig(
                *threshold,
                members,
                *include_signer,
                client,
                signer.as_ref(),
                &matches,
            )?;
        },
        Command::ProposeMemo {
            multisig_wallet,
            memo,
        } => {
            propose_memo(
                multisig_wallet,
                memo,
                client,
                signer,
            )?;
        },
        Command::Approve { transaction} => {
            approve(
                transaction,
                client,
                signer,
            )?;
        },
        Command::Unapprove { transaction} => {
            unapprove(
                transaction,
                client,
                signer,
            )?;
        },
        Command::Execute { transaction} => {
            execute(
                transaction,
                client,
                signer,
            )?;
        },
        Command::ProposeNewThreshold {
            multisig_wallet,
            threshold,
        } => {
            propose_new_threshold(
                multisig_wallet,
                *threshold,
                client,
                signer,
            )?;
        },
        Command::ProposeNewMembers {
            multisig_wallet,
            members,
        } => {
            let app = Opts::into_app();
            let matches = app.get_matches();
            propose_new_members(
                multisig_wallet,
                members.clone(),
                &matches,
                client,
                signer,
            )?;
        },
    }
    Ok(())
}

pub fn new_multisig(
    threshold: u16,
    members: &Vec<String>,
    include_signer: bool,
    client: RpcClient,
    payer: &dyn Signer,
    matches: &ArgMatches,
) -> Result<()> {
    let mut members: Vec<Pubkey> = members
        .iter()
        .map(|path| pubkey_or_signer_path(path, matches))
        .flatten()
        .collect();
    if include_signer {
        members.push(payer.pubkey());
    }
    let base = Keypair::new();
    let multisig_address = find_multisig_wallet_address(&base.pubkey());
    println!("Creating multisig wallet: {}", multisig_address.to_string());
    let signature = new_multisig_rpc(
        threshold,
        members.clone(),
        &client,
        payer,
        Some(&base),
    )?;
    println!("New multisig successfully created. \
    signature: {}", signature.to_string());
    Ok(())
}

pub fn propose_memo(
    multisig_wallet: &Pubkey,
    memo: &str,
    client: RpcClient,
    signer: Box<dyn Signer>,
) -> Result<()> {
    let memo_ix = spl_memo::build_memo(
        memo.as_ref(), &[multisig_wallet]);
    let mut member = MultisigMember::try_new(
        signer,
        multisig_wallet.clone(),
        client,
    )?;
    let proposal = member.next_transaction_pubkey();
    println!("Creating transaction proposal: {}", proposal.to_string());
    let signature = member.create_and_approve_tx(vec![memo_ix])?;
    println!("New transaction proposal successfully created. \
    signature: {}", signature.to_string());
    Ok(())
}

pub fn approve(
    transaction: &Pubkey,
    client: RpcClient,
    signer: Box<dyn Signer>
) -> Result<()> {
    let act_data = client.get_account_data(transaction)?;
    let tx_data = MultisigTransaction::try_deserialize(&mut act_data.as_slice())?;
    let member = MultisigMember::try_new(
        signer,
        tx_data.multisig_wallet.clone(),
        client,
    )?;
    let signature = member.approve_rpc(transaction.clone())?;
    println!("Transaction successfully approved. \
    signature: {}", signature.to_string());
    Ok(())
}

pub fn unapprove(
    transaction: &Pubkey,
    client: RpcClient,
    signer: Box<dyn Signer>
) -> Result<()> {
    let act_data = client.get_account_data(transaction)?;
    let tx_data = MultisigTransaction::try_deserialize(&mut act_data.as_slice())?;
    let member = MultisigMember::try_new(
        signer,
        tx_data.multisig_wallet.clone(),
        client,
    )?;
    let signature = member.unapprove_rpc(transaction.clone())?;
    println!("Transaction approval succesfully cancelled. \
    signature: {}", signature.to_string());
    Ok(())
}

pub fn execute(
    transaction: &Pubkey,
    client: RpcClient,
    signer: Box<dyn Signer>
) -> Result<()> {
    let act_data = client.get_account_data(transaction)?;
    let tx_data = MultisigTransaction::try_deserialize(&mut act_data.as_slice())?;
    let member = MultisigMember::try_new(
        signer,
        tx_data.multisig_wallet.clone(),
        client,
    )?;
    let signature = member.execute_rpc(transaction.clone())?;
    println!("Transaction succesfully executed. \
    signature: {}", signature.to_string());
    // Hard-coded localnet explorer URL, because this is a demo!
    println!("https://explorer.solana.com/tx/{}?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899",
        signature.to_string());
    Ok(())
}

pub fn propose_new_threshold(
    multisig_wallet: &Pubkey,
    threshold: u16,
    client: RpcClient,
    signer: Box<dyn Signer>,
) -> Result<()> {
    let member = MultisigMember::try_new(
        signer,
        multisig_wallet.clone(),
        client,
    )?;
    println!("New threshold: {}", threshold);
    println!("Creating transaction proposal: {}", member.next_transaction_pubkey().to_string());
    let signature = member.propose_change_threshold(threshold)?;
    println!("New transaction proposal successfully created. \
    signature: {}", signature.to_string());
    Ok(())
}

pub fn propose_new_members(
    multisig_wallet: &Pubkey,
    members: Vec<String>,
    matches: &ArgMatches,
    client: RpcClient,
    signer: Box<dyn Signer>,
) -> Result<()> {
    let members: Vec<Pubkey> = members
        .iter()
        .map(|path| pubkey_or_signer_path(path, matches))
        .flatten()
        .collect();
    let member = MultisigMember::try_new(
        signer,
        multisig_wallet.clone(),
        client,
    )?;
    println!("New members: {:?}", members);
    println!("Creating transaction proposal: {}", member.next_transaction_pubkey().to_string());
    let signature = member.propose_change_members(members)?;
    println!("New transaction proposal successfully created. \
    signature: {}", signature.to_string());
    Ok(())
}
