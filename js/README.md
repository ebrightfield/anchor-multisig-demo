## Demo Multisig SDK

This SDK provides a wrapper over the
Anchor `Program` object to make it easier to work
with this multisig program. Rote implementation details
are factored out of the interface, leaving simpler argument signatures.

It revolves around a general concept of combining:
1. A signer.
2. A chosen multisig wallet of which (1) is a member.

The `MultisigMember` class combines the above-mentioned elements and exposes methods
for the creation of transaction instructions, as well as performing direct RPC calls.
There are convenience functions for calculating PDAs as well.
