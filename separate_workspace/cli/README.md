# Multisig Demo CLI

This CLI piggybacks off of the same CLI configuration interface
used in the official Solana CLI binaries.

The `-k/--keypair` and `-u/--url` arguments behave the same way, as well.
For example, you can pass `-k prompt://` or `-k usb://ledger?key=0` as signer paths.
Similarly, the `-u` takes the shortcut strings like `-ul`, `-u devnet`, etc.

In addition, the CLI will fallback to whatever is configured in `~/.config/solana/cli/config.yml`.
This allows one to take advantage of `solana config set` to persistently configure
their target cluster and signer path over numerous commands.

The CLI uses the Clap library and attempts to be as self-documenting
as possible with `--help` flags.

```
$ clap build -p msig-cli
$ target/debug/msig-cli --help
```
