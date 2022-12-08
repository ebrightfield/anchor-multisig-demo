#!/bin/sh

set -x

# NOTE: This script requires a localnet running with the program.
# This is conveniently achieved by executing `anchor test --detach`.

# Execute this from up a level,
# i.e. $ ./examples/demo.sh

solana config set -ul -k ../tests/accounts/test_user1-keypair.json

echo "First we will make a new multisig group (with test users 1,2,3 as members):"

# Make a new multisig
cargo run -p msig-cli new-multisig \
    --include-signer \
    --threshold 2 \
    ../tests/accounts/test_user2-keypair.json \
    ../tests/accounts/test_user3-keypair.json

echo "Copy-paste the new multisig account, and press ENTER: "
read MSIG

echo "User 2 will now propose a memo and approve it..."

target/debug/msig-cli \
    -k ../tests/accounts/test_user2-keypair.json \
    propose-memo \
    $MSIG \
    "hello from the command-line"

echo "Copy-paste the new transaction proposal account, and press ENTER: "
read TX

echo "User 3 Approving..."

target/debug/msig-cli \
    -k ../tests/accounts/test_user3-keypair.json \
    approve \
    $TX

echo "Sleeping to let the validator catch up..."
sleep 2

echo "User 1 Executing"

target/debug/msig-cli \
    execute \
    $TX
