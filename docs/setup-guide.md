# Setup Guide

## Prerequisites

- **dfx** (ICP SDK) — version 0.24.x or later
  ```bash
  curl -sSL https://sdk.dfinity.org/install.sh | sh
  ```

- **Rust** — stable toolchain with wasm32 target
  ```bash
  rustup default stable
  rustup target add wasm32-unknown-unknown
  ```

- **Node.js** 20+ — for the frontend
  ```bash
  node --version  # should be v20+
  ```

## Quick Start (Local)

```bash
git clone https://github.com/YOUR_USERNAME/zeroclaw-icp.git
cd zeroclaw-icp

# Run the local deploy script
chmod +x scripts/deploy-local.sh
./scripts/deploy-local.sh
```

This will:
1. Build the vault and agent canisters (Rust → Wasm32)
2. Start a local ICP replica
3. Deploy both canisters
4. Print canister IDs

## Building the Frontend

```bash
cd canisters/frontend

# Copy and edit environment config
cp .env.example .env
# Edit .env and set VITE_AGENT_CANISTER_ID to your agent's local ID

# Install dependencies
npm install

# Build for local
npm run build

# Deploy frontend
dfx deploy frontend --network local
```

## Testing the Agent

```bash
# Check cycles balance
dfx canister call agent cycles_balance

# Send a chat message
dfx canister call agent chat '("Hello, ZeroClaw!")'

# Check conversation history
dfx canister call agent get_history '(50)'

# Add a scheduled SOP
dfx canister call agent add_sop '("daily-briefing", "0 9 * * *", "Give me a summary of my tasks for today", "Morning briefing")'

# List all SOPs
dfx canister call agent list_sops

# Check economics stats
dfx canister call agent get_economics_stats

# Check operational mode
dfx canister call agent operational_mode
```

## Mainnet Deployment

```bash
# Ensure you have ICP in your dfx identity wallet
dfx identity get-principal
dfx identity get-balance

# Deploy to mainnet
chmod +x scripts/deploy-mainnet.sh
./scripts/deploy-mainnet.sh
```

## Connecting Vault to Agent

After deploying both canisters:

```bash
# Get vault's controller principal (the deployer)
dfx identity get-principal

# Set the agent principal in vault (run this once)
dfx canister call vault set_agent_principal "(principal \"<AGENT_PRINCIPAL>\")"

# Now the agent can fetch API keys from vault
dfx canister call agent whoami  # should match the agent principal you set
```

## Storing API Keys

```bash
# Store an API key (encrypted value as hex or base64 blob)
dfx canister call vault store_key '("anthropic", vec { 0x3a; 0xb2; ... })'

# Agent retrieves key when making HTTPS outcalls
# (this happens automatically in the provider layer)
```

## Environment Variables

| Variable | Description | Default |
|---|---|---|
| `VITE_AGENT_CANISTER_ID` | Agent canister ID for frontend | `ryjl3-tyaaa-aaaaa-aaaba-cai` |
| `DFX_NETWORK` | Network to deploy to | `local` |

## Troubleshooting

**dfx not found**: Restart your shell after installing, or add `~/.local/bin` to PATH.

**wasm32 target not found**: Run `rustup target add wasm32-unknown-unknown`

**Replica won't start**: Kill any existing dfx processes: `dfx stop && dfx start --background`

**Frontend can't reach agent**: Ensure `VITE_AGENT_CANISTER_ID` matches the local canister ID printed by deploy-local.sh

**cycles_balance returns 0**: The agent needs cycles. On mainnet: `dfx wallet send --network ic --canister <AGENT_ID> <amount>`

## Common dfx Commands

```bash
dfx start              # Start local replica (foreground)
dfx stop               # Stop local replica
dfx deploy             # Deploy all canisters
dfx canister status     # Check canister status
dfx canister id         # Get canister ID
dfx build              # Build canisters
dfx generate           # Generate type bindings
```
