## Sandboxed Cargo Workspace

This is a simple workaround to a recent semver bug with `anchor-client`
in workspaces where Anchor 0.25.0 programs also reside.

It contains:
1. The Rust SDK
2. A Rust CLI

For a demo of the CLI, including the full flow from creation, approval, and execution:
1. Run `anchor test --detach` in another terminal.
2. Then run `examples/demo.sh`.
