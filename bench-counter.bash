#!/usr/bin/env bash
# What a hook with no Solidity in it costs, against the same hook in Solidity.
#
#   ./bench-counter.bash
#
# Runs the same counter hook two ways on a chain that can execute both EVM bytecode and WASM:
#
#   Counter.sol     one Solidity contract
#   native-counter  a Stylus contract that IS the hook, no Solidity anywhere
#
# A counter is the shape of hook Stylus is worst at -- it reads a number, adds to it and writes it
# back, which is all storage and no arithmetic. That is the point: this is the floor.
#
# One swap per transaction, so the cost of each comes off the receipt.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-counter-bench}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
ARB_OWNER=0x0000000000000000000000000000000000000070
ARB_WASM=0x0000000000000000000000000000000000000071
EOA=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E
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

log "deploying v4 and the Solidity hook"
(cd "$ROOT/uniswap" && CREATE2_DEPLOYER="$CREATE2_DEPLOYER" \
  forge script script/bench/DeployCounterBench.s.sol:DeployCounterBenchScript \
    --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
    --gas-estimate-multiplier 200 --broadcast --slow) >"$WORK/deploy.log" 2>&1 \
  || { tail -40 "$WORK/deploy.log"; exit 1; }

addr_of() { grep -m1 "^  $1" "$WORK/deploy.log" | grep -oE '0x[0-9a-fA-F]{40}' || true; }
STYLUS_DEPLOYER=$(addr_of "stylusDeployer")
POOL_MANAGER=$(addr_of "poolManager")
# The Rust hook holds the pool manager as a build-time constant rather than in storage, so it costs
# nothing to read on every callback. Its constructor still takes the address, purely to check it
# against what was compiled in — a wrong $POOL_MANAGER then fails the deployment instead of
# producing a hook that silently rejects every call. See stylus/native-counter/build.rs.
export POOL_MANAGER="$POOL_MANAGER"
CURRENCY0=$(addr_of "currency0")
CURRENCY1=$(addr_of "currency1")
FIXTURE=$(addr_of "fixture")
SOLIDITY_HOOK=$(addr_of "Counter (sol)")
grep -E "^  [a-zA-Z].*0x" "$WORK/deploy.log" | sed 's/^  //'

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
log "opening one pool per variant"
open_pool() {
  cast send "$FIXTURE" "open(address,address,address,uint128)" \
    "$CURRENCY0" "$CURRENCY1" "$1" "$LIQUIDITY" --rpc-url "$RPC" --private-key $KEY \
    | awk '/^gasUsed/{print $2}'
}
G_NONE=$(open_pool 0x0000000000000000000000000000000000000000)
G_SOL=$(open_pool "$SOLIDITY_HOOK")
G_NATIVE=$(open_pool "$NATIVE_HOOK")
printf 'add liquidity: none=%s solidity=%s native=%s\n' "$G_NONE" "$G_SOL" "$G_NATIVE"

log "swapping ($ROUNDS rounds; the first is cold)"
printf '%-8s %12s %12s %12s\n' round "no hook" "solidity" "native"
declare -a LAST
for round in $(seq 1 "$ROUNDS"); do
  row=""
  for i in 0 1 2; do
    g=$(cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$i" "$SWAP_AMOUNT" true \
      --rpc-url "$RPC" --private-key $KEY | awk '/^gasUsed/{print $2}')
    LAST[$i]=$g
    row=$(printf '%s %12s' "$row" "$g")
  done
  printf '%-8s%s\n' "$round" "$row"
done

UNCACHED_NATIVE=${LAST[2]}

log "caching the Stylus program, and swapping again"
# Every other benchmark here reports the cached figure, because that is the state any hook with
# users would be in: caching is a one-off bid, and an uncached program pays the full WASM load on
# every call forever. Measured rather than projected -- the dev node has no CacheManager, so the
# chain owner appoints itself one. See `cache_stylus_program` in bench-lib.bash.
cached=$(cache_stylus_program "$NATIVE_HOOK" "$EOA")
echo "native hook cached: $cached"
for i in 0 1 2; do
  LAST[$i]=$(cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$i" "$SWAP_AMOUNT" true \
    --rpc-url "$RPC" --private-key $KEY | awk '/^gasUsed/{print $2}')
done

log "results (gas per swap — two hook calls each: beforeSwap and afterSwap)"
base=${LAST[0]}; sol=${LAST[1]}; native=${LAST[2]}
printf 'baseline swap, no hook                  %10s\n' "$base"
printf 'Counter.sol, one Solidity contract      %10s  (+%s)\n' "$sol" "$((sol - base))"
printf 'native-counter, uncached                %10s  (+%s)\n' \
  "$UNCACHED_NATIVE" "$((UNCACHED_NATIVE - base))"
printf 'native-counter, cached                  %10s  (+%s)\n' "$native" "$((native - base))"
echo
printf 'the native hook costs, over Solidity    %10s\n' "$((native - sol))"
echo
printf 'per-call Stylus program init, %s bytes of asm\n' \
  "$(cast call $ARB_WASM 'codehashAsmSize(bytes32)(uint32)' "$(cast codehash "$NATIVE_HOOK" --rpc-url "$RPC")" --rpc-url "$RPC" | awk '{print $1}')"
printf '  uncached                              %10s\n' "$NATIVE_INIT"
printf '  cached, which is what is measured     %10s\n' "$NATIVE_INIT_CACHED"
echo
echo "A counter is the shape of hook Stylus is worst at: it writes a storage slot per callback and"
echo "computes nothing, and Stylus makes computation cheap, not storage. This is the floor."
