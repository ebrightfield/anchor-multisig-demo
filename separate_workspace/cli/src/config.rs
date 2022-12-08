use anyhow::{anyhow, Result};
use clap::parser::ArgMatches;
use solana_clap_v3_utils::keypair::signer_from_path;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use solana_sdk::signature::Signer;
use clap::Parser;
use solana_cli_config::Config;

/// Put this (flattened) at the top level of a Clap CLI made with the Derive API to add the
/// `-u/--url` CLI arg as it functions in the official Solana CLI.
/// This allows for manual specification of a cluster url,
/// or otherwise defaulting to the Solana CLI config file.
#[derive(Debug, Parser)]
pub struct UrlArg {
    /// The target URL for the cluster. See Solana CLI documentation on how to use this.
    /// Default values and usage patterns are identical to Solana CLI.
    #[clap(short, long)]
    pub url: Option<String>,
}

impl UrlArg {
    pub fn resolve(&self, config: Option<&Config>) -> Result<String> {
        if let Some(url) = self.url.clone() {
            return Ok(url);
        }
        if let Some(config) = config {
            return Ok(config.json_rpc_url.clone());
        }
        let config = get_solana_cli_config()?;
        return Ok(config.json_rpc_url);
    }
}

/// Put this (flattened) at the top level of a Clap CLI made with the Derive API to add the
/// `-k/--keypair` CLI arg as it functions in the Solana CLI.
/// This allows for manual specification of a signing keypair,
/// or otherwise defaulting to the Solana CLI config file.
/// `--skip_phrase_validation` and `--confirm-key` are necessary because
/// signer resolution uses [solana_clap_v3_utils::keypair::signer_from_path].
#[derive(Debug, Parser)]
pub struct KeypairArg {
    /// The target signer for transactions. See Solana CLI documentation on how to use this.
    /// Default values and usage patterns are identical to Solana CLI.
    #[clap(short, long)]
    pub keypair: Option<String>,
    /// Skip BIP-39 seed phrase validation (not recommended)
    #[clap(long, name = "skip_seed_phrase_validation")]
    pub skip_seed_phrase_validation: bool,
    /// Manually confirm the signer before proceeding
    #[clap(long, name = "confirm_key")]
    pub confirm_key: bool,
}

impl KeypairArg {
    pub fn resolve(&self,
                   matches: &ArgMatches,
                   config: Option<&Config>,
    ) -> Result<Box<dyn Signer>> {
        if let Some(keypair_path) = self.keypair.clone() {
            return parse_signer(matches, keypair_path.as_str());
        }
        if let Some(config) = config {
            return parse_signer(matches, &config.keypair_path);
        }
        let config = get_solana_cli_config()?;
        parse_signer(matches, &config.keypair_path)
    }
}

/// Parses [solana_sdk::pubkey::Pubkey] from a string.
pub fn pubkey_arg(pubkey: &str) -> Result<Pubkey> {
    Pubkey::from_str(pubkey).map_err(
        |e| anyhow!("invalid pubkey: {}", e.to_string())
    )
}

/// Returns a pubkey using either its string representation,
/// or reading it as a signer path and retaining only that signer's public key.
/// Useful when you want a pubkey, but it might be more convenient to pass
/// a signer path.
pub fn pubkey_or_signer_path(input: &str, matches: &ArgMatches) -> Result<Pubkey> {
    if let Ok(pubkey) = Pubkey::from_str(input) {
        Ok(pubkey)
    } else {
        let mut wallet_manager = None;
        let signer = signer_from_path(
            matches,
            input,
            "keypair",
            &mut wallet_manager,
        ).map_err(
            |e| anyhow!("invalid pubkey or signer path {}: {}", input, e.to_string())
        )?;
        Ok(signer.pubkey())
    }
}

/// Branch over the possible ways that signers can be specified via user input.
/// This basically does what `-k/--keypair` does, on a specific input string,
/// with disregard to filesystem configuration. It is useful for situations
/// where additional signers may be specified, e.g. grinding for an address and using
/// it as a signer when creating a multisig account.
pub fn parse_signer(matches: &ArgMatches, path: &str) -> Result<Box<dyn Signer>> {
    let mut wallet_manager = None;
    let signer = signer_from_path(
        matches,
        path,
        "keypair",
        &mut wallet_manager,
    ).map_err(|e| anyhow!("Could not resolve signer: {:?}", e))?;
    Ok(signer)
}

/// Load configuration from the standard Solana CLI config path.
/// Those config values are used as defaults at runtime whenever
/// keypair and/or url are not explicitly passed in.
/// This can possibly fail if there is no Solana CLI installed, nor a config file
/// at the expected location.
pub fn get_solana_cli_config() -> anyhow::Result<Config> {
    let config_file = solana_cli_config::CONFIG_FILE.as_ref()
        .ok_or_else(|| anyhow!("unable to determine a config file path on this OS or user"))?;
    Config::load(&config_file)
        .map_err(|e| anyhow!("unable to load config file: {}", e.to_string()))
}