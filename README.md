## Solana Multisig Program Demonstration

This repo is for demonstration purposes only, and was created as part of a skills demo.
It contains a multisig program of the usual "M of N" variant.
For a production ready multisig program, please see the [Goki Smart Wallet](https://github.com/GokiProtocol/goki/tree/master/programs/smart-wallet) program instead.

Skills Demonstrated:
- Solana program development using the Anchor framework.
- SDK development in both Rust and Typescript.
- Advanced peripheral tooling for extensive and at-scale testing.
- Command-line utilities for direct program interaction with familiar Solana CLI ergonomics.
- Clearly written and thorough documentation.

In addition to what is expected of an M of N multisig program,
other notable features include:
- Multisig transaction proposals can contain >1 instruction.
- There is an "unapprove" instruction that allows cancellation of an approval.
- Transaction creation, execution, and approval history are retained
  (approvals are stored as `Option<i64>` unix timestamps instead of booleans).
- Multisig Wallets and Transactions are coordinated with owner set sequence numbers.
This prevents re-ordering attacks and other complications with allowing mutation of
the owner set or threshold.


## Project Structure
- The program itself is in the usual `programs/` directory.
- `js/` contains the Typescript SDK.
- `separate_workspace/sdk/` contains the Rust SDK.
- `separate_workspace/cli/` contains the Rust CLI.
- `tests/` is configured via its own `Test.toml` separate from the project's central `Anchor.toml`.

#### Why the Separate Workspace?
There is a recent semver related bug where if a workspace contains Anchor 0.25.0 programs,
then it cannot contain other crates that use `anchor-client` 0.25.0.

#### Why the copy of `solana-clap-v3-utils`?
Anchor's latest release (0.25.0) cannot take Solana dependencies 1.11 or higher.
`solana-clap-v3-utils` only exists on Solana ^1.11. This local copy modifies
the versioning to be compatible with Solana 1.10.X.

## Initial Setup
Before running `anchor test`, remember that you'll need to build the TS SDK
and other test dependencies.
You'll need to run something along the lines of:
```
$ cd js && yarn install && yarn run build && cd - && yarn install
```

## Testing
A simple `anchor test` should execute the Typescript integration tests.

To execute Rust unit tests, `cargo test` in the respective workspace roots will work as expected.
