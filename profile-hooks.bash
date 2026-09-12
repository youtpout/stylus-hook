#!/usr/bin/env bash
# Which shipping Uniswap hooks are worth rewriting in Rust?
#
#   ./profile-hooks.bash
#
# Stylus makes arithmetic cheaper — 4x on the mulDiv-class operations Uniswap's own math is built
# from — and leaves storage and calls where they are. So the question for any hook is what fraction
# of its gas is arithmetic. This swaps through one pool per hook and traces the transaction opcode
# by opcode, then subtracts the same swap through a pool with no hook.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-hook-survey}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
ARB_OWNER=0x0000000000000000000000000000000000000070
LIQUIDITY=1000000000000000000000
SWAP_AMOUNT=1000000000000000000

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

log "deploying v4 and the hooks"
(cd "$ROOT/uniswap" && CREATE2_DEPLOYER="$CREATE2_DEPLOYER" \
  forge script script/bench/DeployHookSurvey.s.sol:DeployHookSurveyScript \
    --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
    --gas-estimate-multiplier 200 --broadcast --slow) >"$WORK/deploy.log" 2>&1 \
  || { tail -40 "$WORK/deploy.log"; exit 1; }

addr_of() { grep -m1 "^  $1" "$WORK/deploy.log" | grep -oE '0x[0-9a-fA-F]{40}' || true; }
CURRENCY0=$(addr_of "currency0"); CURRENCY1=$(addr_of "currency1"); FIXTURE=$(addr_of "fixture")
grep -E "^  [a-zA-Z].*0x" "$WORK/deploy.log" | sed 's/^  //'

open_pool() {
  cast send "$FIXTURE" "open(address,address,address,uint128)" \
    "$CURRENCY0" "$CURRENCY1" "$1" "$LIQUIDITY" --rpc-url "$RPC" --private-key $KEY >/dev/null
}
log "opening one pool per hook"
open_pool 0x0000000000000000000000000000000000000000
open_pool "$(addr_of 'LimitOrder')"
open_pool "$(addr_of 'PanopticOracle')"

# Sums the struct log by opcode class. A call opcode's gasCost is the gas handed to the callee, and
# the callee's own opcodes are logged too, so counting it would double up: calls are counted, not
# summed.
profile() {
  local txhash=$1
  cast rpc debug_traceTransaction "$txhash" \
    '{"disableStack":true,"disableMemory":true,"disableStorage":true}' --rpc-url "$RPC" \
  | python3 -c '
import sys, json
STORAGE = {"SLOAD","SSTORE","TLOAD","TSTORE"}
CALLS   = {"CALL","STATICCALL","DELEGATECALL","CALLCODE","CREATE","CREATE2"}
d = json.load(sys.stdin)
acc = {"compute":0,"storage":0,"keccak":0,"log":0,"calls":0}
for s in d["structLogs"]:
    op, cost = s["op"], s.get("gasCost",0)
    if op in CALLS:      acc["calls"] += 1
    elif op in STORAGE:  acc["storage"] += cost
    elif op == "KECCAK256" or op == "SHA3": acc["keccak"] += cost
    elif op.startswith("LOG"): acc["log"] += cost
    else:                acc["compute"] += cost
print(acc["compute"], acc["storage"], acc["keccak"], acc["log"], acc["calls"], d["gas"])
'
}

swap_tx() {
  cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$1" "$SWAP_AMOUNT" "$2" \
    --rpc-url "$RPC" --private-key $KEY --json | python3 -c 'import sys,json;print(json.load(sys.stdin)["transactionHash"])'
}

log "warming every pool"
for i in 0 1 2; do swap_tx "$i" true >/dev/null; swap_tx "$i" false >/dev/null; done

log "profiling one swap through each"
declare -A NAME=( [0]="no hook" [1]="LimitOrder" [2]="PanopticOracle" )
printf '%-16s %10s %10s %10s %8s %10s\n' hook compute storage keccak calls total
declare -a C S K
for i in 0 1 2; do
  read -r c s k l n g <<<"$(profile "$(swap_tx "$i" false)")"
  C[$i]=$c; S[$i]=$s; K[$i]=$k
  printf '%-16s %10s %10s %10s %8s %10s\n' "${NAME[$i]}" "$c" "$s" "$k" "$n" "$g"
done

log "what each hook adds, over the same swap with no hook"
printf '%-16s %10s %10s %10s %10s\n' hook compute storage keccak "compute %"
for i in 1 2; do
  dc=$(( C[i] - C[0] )); ds=$(( S[i] - S[0] )); dk=$(( K[i] - K[0] ))
  tot=$(( dc + ds + dk ))
  pct=0; [ "$tot" -gt 0 ] && pct=$(( dc * 100 / tot ))
  printf '%-16s %10s %10s %10s %9s%%\n' "${NAME[$i]}" "$dc" "$ds" "$dk" "$pct"
done
echo
echo "Only the compute column moves in Stylus, and it moves by about 2.8x on the mulDiv-class"
echo "arithmetic Uniswap is built from. The bar for a port to pay for itself is roughly 62,000"
echo "gas of arithmetic per call — see BENCHMARK.md."
