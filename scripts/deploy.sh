#!/usr/bin/env bash
# Deploy danfo-registry and danfo-rewards to Stellar testnet, in order:
#   1. build both wasms (registry first — rewards imports its interface)
#   2. deploy + init registry (slash_recipient temporarily = admin)
#   3. deploy + init rewards (pointing at registry)
#   4. registry.set_config -> slash_recipient = rewards (spam tops up the pool)
#
# Prereqs: rust + wasm32-unknown-unknown target + stellar CLI.
# Usage:   ./scripts/deploy.sh
#
# Tunables (env): STELLAR_NETWORK, STELLAR_IDENTITY, STAKE_AMOUNT,
# CHALLENGE_WINDOW, MIN_VOTES, REWARD_AMOUNT.
set -euo pipefail
cd "$(dirname "$0")/.."

NET="${STELLAR_NETWORK:-testnet}"
IDENT="${STELLAR_IDENTITY:-danfo}"
STAKE="${STAKE_AMOUNT:-100000000}"        # 10 XLM in stroops
WINDOW="${CHALLENGE_WINDOW:-86400}"       # 24h in seconds
MIN_VOTES="${MIN_VOTES:-2}"
REWARD="${REWARD_AMOUNT:-50000000}"       # 5 XLM in stroops

echo "[1/6] ensure network + identity ($NET / $IDENT)…"
stellar network add "$NET" \
  --rpc-url "${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}" \
  --network-passphrase "Test SDF Network ; September 2015" 2>/dev/null || true
stellar keys generate "$IDENT" --network "$NET" --fund 2>/dev/null || \
  stellar keys fund "$IDENT" --network "$NET" 2>/dev/null || true
ADMIN=$(stellar keys address "$IDENT")
echo "  admin address: $ADMIN"

echo "[2/6] build wasms (registry first)…"
cargo build -p danfo-registry --release --target wasm32-unknown-unknown
cargo build -p danfo-rewards  --release --target wasm32-unknown-unknown

REG_WASM=target/wasm32-unknown-unknown/release/danfo_registry.wasm
REW_WASM=target/wasm32-unknown-unknown/release/danfo_rewards.wasm
stellar contract optimize --wasm "$REG_WASM" 2>/dev/null || true
stellar contract optimize --wasm "$REW_WASM" 2>/dev/null || true
[ -f "${REG_WASM%.wasm}.optimized.wasm" ] && REG_WASM="${REG_WASM%.wasm}.optimized.wasm"
[ -f "${REW_WASM%.wasm}.optimized.wasm" ] && REW_WASM="${REW_WASM%.wasm}.optimized.wasm"

echo "[3/6] XLM SAC id…"
TOKEN=$(stellar contract id asset --asset native --network "$NET")
echo "  token: $TOKEN"

echo "[4/6] deploy + init registry…"
REGISTRY=$(stellar contract deploy --wasm "$REG_WASM" --source "$IDENT" --network "$NET")
stellar contract invoke --id "$REGISTRY" --source "$IDENT" --network "$NET" -- \
  init --config "{\"admin\":\"$ADMIN\",\"token\":\"$TOKEN\",\"stake_amount\":\"$STAKE\",\"challenge_window\":$WINDOW,\"min_votes\":$MIN_VOTES,\"slash_recipient\":\"$ADMIN\"}"

echo "[5/6] deploy + init rewards…"
REWARDS=$(stellar contract deploy --wasm "$REW_WASM" --source "$IDENT" --network "$NET")
stellar contract invoke --id "$REWARDS" --source "$IDENT" --network "$NET" -- \
  init --config "{\"admin\":\"$ADMIN\",\"token\":\"$TOKEN\",\"registry\":\"$REGISTRY\",\"reward_amount\":\"$REWARD\"}"

echo "[6/6] point registry slash_recipient at rewards pool…"
stellar contract invoke --id "$REGISTRY" --source "$IDENT" --network "$NET" -- \
  set_config --new_config "{\"admin\":\"$ADMIN\",\"token\":\"$TOKEN\",\"stake_amount\":\"$STAKE\",\"challenge_window\":$WINDOW,\"min_votes\":$MIN_VOTES,\"slash_recipient\":\"$REWARDS\"}"

echo
echo "=========================================================="
echo " Deployed. Add to danfo-app environment:"
echo
echo "   NEXT_PUBLIC_REGISTRY_CONTRACT=$REGISTRY"
echo "   NEXT_PUBLIC_REWARDS_CONTRACT=$REWARDS"
echo "   NEXT_PUBLIC_STAKE_TOKEN=$TOKEN"
echo "=========================================================="
