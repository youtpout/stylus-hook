#!/usr/bin/env bash
# What does OpenZeppelin's AntiSandwichHook actually cost, and how much of that could Stylus take?
#
#   ./bench-antisandwich.bash
#
# The hook checkpoints the pool at the top of each block and, for `zeroForOne == false` swaps,
# replays Pool.swap against that checkpoint so the trade cannot get a better price than the block
# started with. `zeroForOne == true` swaps skip the replay. So the gap between the two directions is
# the AMM simulation on its own — the part that is arithmetic, and the only part a Rust port could
# make cheaper.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-antisandwich-bench}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
ARB_OWNER=0x0000000000000000000000000000000000000070
LIQUIDITY=1000000000000000000000
SWAP_AMOUNT=1000000000000000000
ROUNDS=${ROUNDS:-3}

ROOT=$(cd "$(dirname "$0")" && pwd)
WORK=$(mktemp -d)
export PATH="$HOME/.foundry/bin:$PATH"

log() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

log "starting a fresh nitro dev node"
docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
docker run --rm -d --name "$CONTAINER" -p 8547:8547 "$NITRO_IMAGE" \
  --dev --http.addr 0.0.0.0 --http.port 8547 --http.api=net,web3,eth,debug,arb \
  --http.corsdomain='*' --http.vhosts='*' >/dev/null
trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true; rm -rf "$WORK"' EXIT
for _ in $(seq 1 60); do cast chain-id --rpc-url "$RPC" >/dev/null 2>&1 && break; sleep 1; done

cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 --rpc-url "$RPC" --private-key $KEY >/dev/null
CREATE2_DEPLOYER=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3 \
  | awk '/^contractAddress/{print $2}')

# AntiSandwichHook keys its checkpoint on `block.number`. On Arbitrum that is an estimate of the
# *L1* block number, not the L2 one, so check what this chain reports before trusting the result.
# runtime: NUMBER PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
PROBE=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x6009600c60003960096000f34360005260206000f3 | awk '/^contractAddress/{print $2}')
log "what this chain calls a block"
printf 'solidity block.number      %s\n' "$(cast call "$PROBE" --rpc-url "$RPC" | cast to-dec)"
printf 'ArbSys.arbBlockNumber()    %s\n' "$(cast call 0x0000000000000000000000000000000000000064 'arbBlockNumber()(uint256)' --rpc-url "$RPC" | awk '{print $1}')"
printf 'eth_blockNumber            %s\n' "$(cast block-number --rpc-url "$RPC")"

log "deploying v4 and OpenZeppelin's AntiSandwichHook"
(cd "$ROOT/uniswap" && CREATE2_DEPLOYER="$CREATE2_DEPLOYER" \
  forge script script/bench/DeployAntiSandwichBench.s.sol:DeployAntiSandwichBenchScript \
    --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
    --gas-estimate-multiplier 200 --broadcast --slow) >"$WORK/deploy.log" 2>&1 \
  || { tail -40 "$WORK/deploy.log"; exit 1; }

addr_of() { grep -m1 "^  $1" "$WORK/deploy.log" | grep -oE '0x[0-9a-fA-F]{40}' || true; }
CURRENCY0=$(addr_of "currency0")
CURRENCY1=$(addr_of "currency1")
FIXTURE=$(addr_of "fixture")
HOOK=$(addr_of "AntiSandwich")
grep -E "^  [a-zA-Z].*0x" "$WORK/deploy.log" | sed 's/^  //'

log "opening a control pool and a hooked pool"
open_pool() {
  cast send "$FIXTURE" "open(address,address,address,uint128)" \
    "$CURRENCY0" "$CURRENCY1" "$1" "$LIQUIDITY" --rpc-url "$RPC" --private-key $KEY >/dev/null
}
open_pool 0x0000000000000000000000000000000000000000
open_pool "$HOOK"

swap_gas() {
  cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$1" "$SWAP_AMOUNT" "$2" \
    --rpc-url "$RPC" --private-key $KEY | awk '/^gasUsed/{print $2}'
}

log "swapping ($ROUNDS rounds; the first is cold)"
printf '%-8s %14s %14s %14s %14s\n' round "none 0->1" "none 1->0" "hook 0->1" "hook 1->0"
for round in $(seq 1 "$ROUNDS"); do
  a=$(swap_gas 0 true); b=$(swap_gas 0 false)
  c=$(swap_gas 1 true); d=$(swap_gas 1 false)
  printf '%-8s %14s %14s %14s %14s\n' "$round" "$a" "$b" "$c" "$d"
done

log "results (warm, gas per swap)"
printf 'baseline swap, no hook          0->1 %10s   1->0 %10s\n' "$a" "$b"
printf 'AntiSandwichHook                0->1 %10s   1->0 %10s\n' "$c" "$d"
echo
printf 'the hook costs, 0->1 (no replay)     %10s\n' "$((c - a))"
printf 'the hook costs, 1->0 (with replay)   %10s\n' "$((d - b))"
printf 'the Pool.swap replay on its own      %10s\n' "$(( (d - b) - (c - a) ))"
echo
echo "Everything outside that last line is checkpointing pool state into the hook's own"
echo "storage and settling an ERC-6909 fee. Storage is ~7% cheaper in Stylus (see"
echo "BENCHMARK.md), so only the replay is worth porting."
