#!/bin/bash
set -e

echo "=== ZeroClaw ICP — Local Deployment ==="

echo "Checking prerequisites..."
command -v dfx >/dev/null 2>&1 || { echo "dfx is required but not installed. See https://internetcomputer.org/docs/current/developer-docs/getting-started/install/" >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "cargo is required but not installed." >&2; exit 1; }
rustup target list --installed | grep wasm32-unknown-unknown >/dev/null 2>&1 || { echo "wasm32 target not installed. Run: rustup target add wasm32-unknown-unknown" >&2; exit 1; }

echo "Building Rust canisters..."
cd "$(dirname "$0")/.."
cargo build --release --manifest-path canisters/vault/Cargo.toml --target wasm32-unknown-unknown 2>/dev/null || cargo build --release --manifest-path canisters/vault/Cargo.toml
cargo build --release --manifest-path canisters/agent/Cargo.toml --target wasm32-unknown-unknown 2>/dev/null || cargo build --release --manifest-path canisters/agent/Cargo.toml

echo "Starting local ICP replica..."
dfx start --background --clean 2>/dev/null || dfx start --background

echo "Waiting for replica to be ready..."
sleep 3

echo "Deploying canisters..."
dfx deploy vault --network local
dfx deploy agent --network local

VAULT_ID=$(dfx canister id vault --network local 2>/dev/null)
AGENT_ID=$(dfx canister id agent --network local 2>/dev/null)

echo ""
echo "=== Deployment Complete ==="
echo "Vault canister ID:  $VAULT_ID"
echo "Agent canister ID:  $AGENT_ID"
echo ""
echo "Frontend: Build the React app with:"
echo "  cd canisters/frontend && npm install && npm run build"
echo "  dfx deploy frontend --network local"
echo ""
echo "Set VITE_AGENT_CANISTER_ID=$AGENT_ID in canisters/frontend/.env before building frontend."