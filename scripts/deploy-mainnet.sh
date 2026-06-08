#!/bin/bash
set -e

echo "=== ZeroClaw ICP — Mainnet Deployment ==="

echo "Checking identity..."
dfx identity whoami >/dev/null 2>&1 || { echo "No dfx identity found. Run: dfx identity new" >&2; exit 1; }
PRINCIPAL=$(dfx identity get-principal 2>/dev/null) || PRINCIPAL="unknown"
echo "Deploying as: $PRINCIPAL"

echo "Building Rust canisters (release)..."
cd "$(dirname "$0")/.."
cargo build --release --manifest-path canisters/vault/Cargo.toml --target wasm32-unknown-unknown
cargo build --release --manifest-path canisters/agent/Cargo.toml --target wasm32-unknown-unknown

echo "Deploying to mainnet..."
read -p "Confirm mainnet deployment? This will spend ICP cycles. (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
fi

dfx deploy vault --network ic
dfx deploy agent --network ic

VAULT_ID=$(dfx canister id vault --network ic 2>/dev/null)
AGENT_ID=$(dfx canister id agent --network ic 2>/dev/null)

echo ""
echo "=== Mainnet Deployment Complete ==="
echo "Your agent is now live forever at:"
echo "  Agent: https://$AGENT_ID.raw.icp0.io"
echo "  Vault: https://$VAULT_ID.raw.icp0.io"
echo ""
echo "Save these canister IDs — your agent lives at these addresses forever."
echo "Your principal: $PRINCIPAL"
echo ""
echo "Add cycles to your agent:"
echo "  dfx canister status $AGENT_ID --network ic"
echo "  dfx wallet send --network ic --canister $AGENT_ID <amount>"