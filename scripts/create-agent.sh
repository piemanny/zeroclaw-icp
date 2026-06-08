#!/bin/bash
set -e

echo "=== ZeroClaw ICP — Create New Agent ==="

AGENT_NAME="${1:-my-agent}"
PROJECT_DIR="$(pwd)/$AGENT_NAME"

if [ -d "$PROJECT_DIR" ]; then
    echo "Error: Directory $AGENT_NAME already exists."
    exit 1
fi

echo "Creating agent '$AGENT_NAME'..."
mkdir -p "$PROJECT_DIR/canisters"
git clone --depth 1 "$(dirname "$0")/.." "$PROJECT_DIR" 2>/dev/null || {
    echo "Error: Not running in zeroclaw-icp directory. Navigate to the zeroclaw-icp root first."
    exit 1
}

cd "$PROJECT_DIR"

echo "Personalizing agent..."
sed -i "s/ryjl3-tyaaa-aaaaa-aaaba-cai/$(dfx canister id agent 2>/dev/null || echo 'YOUR_AGENT_ID')/g" canisters/frontend/.env 2>/dev/null || true

echo ""
echo "Agent '$AGENT_NAME' created at: $PROJECT_DIR"
echo ""
echo "Next steps:"
echo "  cd $AGENT_NAME"
echo "  ./scripts/deploy-local.sh   # for local testing"
echo "  ./scripts/deploy-mainnet.sh # to deploy to mainnet"