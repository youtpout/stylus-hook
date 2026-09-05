#!/usr/bin/env bash
# Does dropping the Solidity shell actually recover the performance it costs?
#
#   ./bench-counter.bash
#
# Runs the same counter hook three ways on a chain that can execute both EVM bytecode and WASM:
#
#   Counter.sol       one Solidity contract
#   CounterProxy.sol  a Solidity shell forwarding to a Stylus contract
#   native-counter    a Stylus contract that IS the hook, no Solidity anywhere
#
# One swap per transaction, so the cost of each comes off the receipt.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-counter-bench}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
ARB_OWNER=0x0000000000000000000000000000000000000070
ARB_WASM=0x0000000000000000000000000000000000000071
LIQUIDITY=1000000000000000000000
SWAP_AMOUNT=1000000000000000000
ROUNDS=${ROUNDS:-3}
FLAGS=before-swap,after-swap,before-add-liquidity,before-remove-liquidity

ROOT=$(cd "$(dirname "$0")" && pwd)
WORK=$(mktemp -d)
export PATH="$HOME/.foundry/bin:$PATH"
# shellcheck source=bench-lib.bash
. "$ROOT/bench-lib.bash"
ensure_binaryen "$WORK"

log() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

log "starting a fresh nitro dev node"
docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
docker run --rm -d --name "$CONTAINER" -p 8547:8547 "$NITRO_IMAGE" \
  --dev --http.addr 0.0.0.0 --http.port 8547 --http.api=net,web3,eth,debug,arb \
  --http.corsdomain='*' --http.vhosts='*' >/dev/null
trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true; rm -rf "$WORK"' EXIT
for _ in $(seq 1 60); do cast chain-id --rpc-url "$RPC" >/dev/null 2>&1 && break; sleep 1; done

# The L1 data component is the same for every variant and swamps what we are comparing.
cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 --rpc-url "$RPC" --private-key $KEY >/dev/null
CREATE2_DEPLOYER=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3 \
  | awk '/^contractAddress/{print $2}')

log "deploying the Stylus contract behind CounterProxy"
STYLUS_COUNTER=$( (cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-counter-hook --no-verify \
  -e "$RPC" --private-key $KEY) 2>&1 | grep -aoE 'deployed code at address[^0]*0x[0-9a-fA-F]{40}' \
  | grep -oE '0x[0-9a-fA-F]{40}' | tail -1)
echo "stylus counter: $STYLUS_COUNTER"

log "deploying v4 and the two Solidity-fronted hooks"
(cd "$ROOT/uniswap" && CREATE2_DEPLOYER="$CREATE2_DEPLOYER" STYLUS_COUNTER="$STYLUS_COUNTER" \
  forge script script/bench/DeployCounterBench.s.sol:DeployCounterBenchScript \
    --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
    --gas-estimate-multiplier 200 --broadcast --slow) >"$WORK/deploy.log" 2>&1 \
  || { tail -40 "$WORK/deploy.log"; exit 1; }

addr_of() { grep -m1 "^  $1" "$WORK/deploy.log" | grep -oE '0x[0-9a-fA-F]{40}' || true; }
STYLUS_DEPLOYER=$(addr_of "stylusDeployer")
POOL_MANAGER=$(addr_of "poolManager")
CURRENCY0=$(addr_of "currency0")
CURRENCY1=$(addr_of "currency1")
FIXTURE=$(addr_of "fixture")
SOLIDITY_HOOK=$(addr_of "Counter (sol)")
PROXY_HOOK=$(addr_of "CounterProxy")
grep -E "^  [a-zA-Z].*0x" "$WORK/deploy.log" | sed 's/^  //'

cast send "$STYLUS_COUNTER" "setHook(address)" "$PROXY_HOOK" --rpc-url "$RPC" --private-key $KEY >/dev/null

log "mining and deploying the hook that has no Solidity at all"
(cd "$ROOT/stylus" && cargo stylus get-initcode --contract stylus-native-counter) 2>/dev/null \
  | tail -1 >"$WORK/initcode.hex"
CONSTRUCTOR=$( (cd "$ROOT/stylus" && cargo stylus constructor --contract stylus-native-counter) 2>/dev/null | tail -1)
(cd "$ROOT/stylus" && cargo run -q -p stylus-hook-miner -- \
  --initcode-file "$WORK/initcode.hex" --permissions "$FLAGS" \
  --constructor-signature "$CONSTRUCTOR" --constructor-args "$POOL_MANAGER" \
  --deployer "$STYLUS_DEPLOYER") >"$WORK/mine.log" 2>&1 || { cat "$WORK/mine.log"; exit 1; }
SALT=$(grep -m1 '^salt:' "$WORK/mine.log" | grep -oE '0x[0-9a-fA-F]{64}')
(cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-native-counter --no-verify \
  -e "$RPC" --private-key $KEY --deployer-address "$STYLUS_DEPLOYER" --deployer-salt "$SALT" \
  --constructor-args "$POOL_MANAGER") >"$WORK/native.log" 2>&1 \
  || { tail -30 "$WORK/native.log"; exit 1; }
NATIVE_HOOK=$(grep -aoiE '(contract deployed at address|deployed code at address)[^0]*0x[0-9a-fA-F]{40}' \
  "$WORK/native.log" | grep -oE '0x[0-9a-fA-F]{40}' | tail -1)
echo "native hook: $NATIVE_HOOK"

read -r NATIVE_INIT NATIVE_INIT_CACHED <<<"$(cast call $ARB_WASM 'programInitGas(address)(uint64,uint64)' \
  "$NATIVE_HOOK" --rpc-url "$RPC" | awk '{print $1}' | tr '\n' ' ')"
read -r PROXIED_INIT PROXIED_INIT_CACHED <<<"$(cast call $ARB_WASM 'programInitGas(address)(uint64,uint64)' \
  "$STYLUS_COUNTER" --rpc-url "$RPC" | awk '{print $1}' | tr '\n' ' ')"

log "opening one pool per variant"
open_pool() {
  cast send "$FIXTURE" "open(address,address,address,uint128)" \
    "$CURRENCY0" "$CURRENCY1" "$1" "$LIQUIDITY" --rpc-url "$RPC" --private-key $KEY \
    | awk '/^gasUsed/{print $2}'
}
G_NONE=$(open_pool 0x0000000000000000000000000000000000000000)
G_SOL=$(open_pool "$SOLIDITY_HOOK")
G_PROXY=$(open_pool "$PROXY_HOOK")
G_NATIVE=$(open_pool "$NATIVE_HOOK")
printf 'add liquidity: none=%s solidity=%s split=%s native=%s\n' "$G_NONE" "$G_SOL" "$G_PROXY" "$G_NATIVE"

log "swapping ($ROUNDS rounds; the first is cold)"
printf '%-8s %12s %12s %12s %12s\n' round "no hook" "solidity" "split" "native"
declare -a LAST
for round in $(seq 1 "$ROUNDS"); do
  row=""
  for i in 0 1 2 3; do
    g=$(cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$i" "$SWAP_AMOUNT" true \
      --rpc-url "$RPC" --private-key $KEY | awk '/^gasUsed/{print $2}')
    LAST[$i]=$g
    row=$(printf '%s %12s' "$row" "$g")
  done
  printf '%-8s%s\n' "$round" "$row"
done

log "results (warm, gas per swap — two hook calls each: beforeSwap and afterSwap)"
base=${LAST[0]}; sol=${LAST[1]}; split=${LAST[2]}; native=${LAST[3]}
printf 'baseline swap, no hook                  %10s\n' "$base"
printf 'Counter.sol, one Solidity contract      %10s  (+%s)\n' "$sol" "$((sol - base))"
printf 'CounterProxy.sol -> Stylus              %10s  (+%s)\n' "$split" "$((split - base))"
printf 'native-counter, no Solidity at all      %10s  (+%s)\n' "$native" "$((native - base))"
echo
printf 'dropping the Solidity shell saves       %10s\n' "$((split - native))"
echo
printf 'per-call Stylus program init, uncached / cached\n'
printf '  native-counter  (%s bytes of asm)  %10s / %s\n' \
  "$(cast call $ARB_WASM 'codehashAsmSize(bytes32)(uint32)' "$(cast codehash "$NATIVE_HOOK" --rpc-url "$RPC")" --rpc-url "$RPC" | awk '{print $1}')" \
  "$NATIVE_INIT" "$NATIVE_INIT_CACHED"
printf '  behind CounterProxy (%s bytes)     %10s / %s\n' \
  "$(cast call $ARB_WASM 'codehashAsmSize(bytes32)(uint32)' "$(cast codehash "$STYLUS_COUNTER" --rpc-url "$RPC")" --rpc-url "$RPC" | awk '{print $1}')" \
  "$PROXIED_INIT" "$PROXIED_INIT_CACHED"
printf 'native still costs, over Solidity       %10s\n' "$((native - sol))"
printf '  Stylus program init, uncached         %10s\n' "$NATIVE_INIT"
printf '  the same if the contract is cached    %10s\n' "$NATIVE_INIT_CACHED"
printf 'projected native cost when cached       %10s  (+%s)\n' \
  "$((native - NATIVE_INIT + NATIVE_INIT_CACHED))" "$((native - NATIVE_INIT + NATIVE_INIT_CACHED - base))"
echo
echo "note: this dev node has no CacheManager, so cached figures come from"
echo "      ArbWasm.programInitGas rather than a measurement."
