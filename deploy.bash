#!/usr/bin/env bash
# Deploys the Stylus airdrop contract and the Uniswap v4 hook that fronts it.
#
#   PRIVATE_KEY=0x... ./deploy.bash [rpc-alias]
#
# `rpc-alias` is a key from uniswap/foundry.toml [rpc_endpoints] (default: arbitrum_sepolia).
set -euo pipefail

RPC_ALIAS="${1:-arbitrum_sepolia}"
case "$RPC_ALIAS" in
  arbitrum_sepolia) ENDPOINT="https://sepolia-rollup.arbitrum.io/rpc" ;;
  arbitrum)         ENDPOINT="https://arb1.arbitrum.io/rpc" ;;
  localhost)        ENDPOINT="http://127.0.0.1:8547" ;;
  *) echo "unknown rpc alias: $RPC_ALIAS" >&2; exit 1 ;;
esac

: "${PRIVATE_KEY:?set PRIVATE_KEY to the deployer's key}"

echo "==> deploying the Stylus contract to $ENDPOINT"
STYLUS_AIRDROP=$(
  cd stylus && cargo stylus deploy \
    --contract stylus-airdrop-hook \
    --endpoint "$ENDPOINT" \
    --private-key "$PRIVATE_KEY" \
    --no-verify 2>&1 | tee /dev/stderr | grep -oiE 'deployed code at address: 0x[0-9a-fA-F]{40}' | grep -oE '0x[0-9a-fA-F]{40}'
)
echo "==> Stylus airdrop contract: $STYLUS_AIRDROP"

echo "==> mining the hook address and deploying the v4 shell"
cd uniswap
STYLUS_AIRDROP="$STYLUS_AIRDROP" forge script script/01_DeployStylusAirdropHook.s.sol \
  --rpc-url "$RPC_ALIAS" \
  --private-key "$PRIVATE_KEY" \
  --broadcast
