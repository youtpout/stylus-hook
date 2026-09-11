#!/usr/bin/env bash
# Measures what the airdrop hook costs per swap, on a chain that can actually execute Stylus.
#
#   ./bench.bash
#
# Stands up a throwaway Arbitrum Nitro dev node in Docker, deploys a full Uniswap v4 stack and four
# pools onto it — one with no hook, one with the hook written in Solidity, one with the hook's state
# in Stylus, and one with the hook's state in a Solidity replica of that same Stylus contract — then
# swaps through each in its own transaction and reads `gasUsed` off the receipts.
#
# The fourth pool is the control: it has the same two-contract shape as the Stylus one, so the gap
# between them is the cost of Stylus itself rather than the cost of the extra call.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-bench}
RPC=${RPC:-http://127.0.0.1:8547}
# the dev account nitro funds in --dev mode
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
DEV=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E
ARB_OWNER=0x0000000000000000000000000000000000000070
ARB_WASM=0x0000000000000000000000000000000000000071
SWAP_AMOUNT=1000000000000000000
ROUNDS=${ROUNDS:-4}

ROOT=$(cd "$(dirname "$0")" && pwd)
export PATH="$HOME/.foundry/bin:$PATH"
# `stylus/airdrop/Stylus.toml` pins a wasm-opt version, so one has to be on PATH before any
# `cargo stylus` call — it refuses to build against a different Binaryen than the one pinned.
BINWORK=$(mktemp -d)
# shellcheck source=bench-lib.bash
. "$ROOT/bench-lib.bash"
ensure_binaryen "$BINWORK"

log() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

log "starting a fresh nitro dev node"
docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
docker run --rm -d --name "$CONTAINER" -p 8547:8547 "$NITRO_IMAGE" \
  --dev --http.addr 0.0.0.0 --http.port 8547 --http.api=net,web3,eth,debug,arb \
  --http.corsdomain='*' --http.vhosts='*' >/dev/null
trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true; rm -rf "$BINWORK"' EXIT

for _ in $(seq 1 60); do
  cast chain-id --rpc-url "$RPC" >/dev/null 2>&1 && break
  sleep 1
done
echo "chain id $(cast chain-id --rpc-url "$RPC"), stylus version $(cast call $ARB_WASM 'stylusVersion()(uint16)' --rpc-url "$RPC")"

# The L1 data-posting component is identical for every variant here — same calldata, same swap — and
# it swamps the L2 execution gas we are trying to compare. Zero it out.
log "zeroing the L1 posting cost"
cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 --rpc-url "$RPC" --private-key $KEY >/dev/null
echo "l1 base fee estimate: $(cast call 0x000000000000000000000000000000000000006C 'getL1BaseFeeEstimate()(uint256)' --rpc-url "$RPC")"

# The canonical deterministic-deployment-proxy cannot be installed from its presigned transaction on
# Arbitrum (its 100k gas limit is below Arbitrum's intrinsic gas), so deploy an identical copy.
log "deploying a CREATE2 factory"
CREATE2_DEPLOYER=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3 \
  | awk '/^contractAddress/{print $2}')
echo "create2 factory: $CREATE2_DEPLOYER"

log "deploying the Stylus airdrop contract"
STYLUS_AIRDROP=$(cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-airdrop-hook --no-verify \
  -e "$RPC" --private-key $KEY 2>&1 | grep -ao 'deployed code at address: .*0x[0-9a-fA-F]\{40\}' \
  | grep -o '0x[0-9a-fA-F]\{40\}' | tail -1)
echo "stylus contract: $STYLUS_AIRDROP"
read -r INIT_GAS INIT_GAS_CACHED <<<"$(cast call $ARB_WASM 'programInitGas(address)(uint64,uint64)' "$STYLUS_AIRDROP" --rpc-url "$RPC" | awk '{print $1}' | tr '\n' ' ')"

log "deploying the v4 stack, the hooks and the pools"
DEPLOY_LOG=$(mktemp)
(cd "$ROOT/uniswap" && CREATE2_DEPLOYER="$CREATE2_DEPLOYER" STYLUS_AIRDROP="$STYLUS_AIRDROP" \
  forge script script/bench/BenchDeploy.s.sol:BenchDeployScript \
    --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
    --gas-estimate-multiplier 200 --broadcast --slow) >"$DEPLOY_LOG" 2>&1 \
  || { tail -40 "$DEPLOY_LOG"; exit 1; }

addr_of() { grep -m1 "^  $1" "$DEPLOY_LOG" | grep -o '0x[0-9a-fA-F]\{40\}'; }
PROXY=$(addr_of "AirdropHookProxy :")
BENCH=$(addr_of "SwapBench")
grep -E "^  [a-zA-Z].*0x" "$DEPLOY_LOG" | sed 's/^  //'

log "binding the Stylus contract to its hook"
# forge cannot do this: an activated Stylus contract's code starts with 0xEF, which the local EVM
# used to simulate scripts rejects as an invalid opcode.
cast send "$STYLUS_AIRDROP" "setHook(address)" "$PROXY" --rpc-url "$RPC" --private-key $KEY >/dev/null
echo "hook: $(cast call "$STYLUS_AIRDROP" 'hook()(address)' --rpc-url "$RPC")"

log "swapping ($ROUNDS rounds; the first is cold, ignore it)"
printf '%-8s %12s %12s %12s %12s\n' round "no hook" "solidity" "stylus" "replica"
declare -a LAST
for round in $(seq 1 "$ROUNDS"); do
  row=""
  for i in 0 1 2 3; do
    g=$(cast send "$BENCH" "swap(uint256,uint256,bool)" "$i" "$SWAP_AMOUNT" true \
      --rpc-url "$RPC" --private-key $KEY | awk '/^gasUsed/{print $2}')
    LAST[$i]=$g
    row=$(printf '%s %12s' "$row" "$g")
  done
  printf '%-8s%s\n' "$round" "$row"
done

log "results (warm, gas per swap)"
base=${LAST[0]}; sol=${LAST[1]}; sty=${LAST[2]}; rep=${LAST[3]}
printf 'baseline swap, no hook                  %10s\n' "$base"
printf 'hook in one Solidity contract           %10s  (+%s)\n' "$sol" "$((sol - base))"
printf 'hook split over two Solidity contracts  %10s  (+%s)\n' "$rep" "$((rep - base))"
printf 'hook with its state in Stylus           %10s  (+%s)\n' "$sty" "$((sty - base))"
echo
printf 'cost of the extra call alone            %10s\n' "$((rep - sol))"
printf 'cost of Stylus over the same Solidity   %10s\n' "$((sty - rep))"
printf '  of which program init (uncached)      %10s\n' "$INIT_GAS"
printf '  program init if the contract is cached%10s\n' "$INIT_GAS_CACHED"
printf 'projected Stylus cost when cached       %10s  (+%s)\n' \
  "$((sty - INIT_GAS + INIT_GAS_CACHED))" "$((sty - INIT_GAS + INIT_GAS_CACHED - base))"
echo
echo "note: this dev node has no CacheManager, so the cached figure is projected from"
echo "      ArbWasm.programInitGas rather than measured."
