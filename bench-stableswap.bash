#!/usr/bin/env bash
# Where does a hook start being cheaper in Rust than in Solidity?
#
#   ./bench-compute.bash
#
# Stylus buys a lower marginal cost of computation at the price of a fixed cost per call. So the
# same hook is cheaper in Solidity below some amount of work and cheaper in Rust above it. This
# sweeps that amount: `ComputeHook.sol` and `stylus/native-compute` run the identical xorshift64
# loop, and the only thing that changes between rows is how many rounds of it.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-stableswap-bench}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
ARB_OWNER=0x0000000000000000000000000000000000000070
ARB_WASM=0x0000000000000000000000000000000000000071
LIQUIDITY=1000000000000000000000
SWAP_AMOUNT=1000000000000000000
ROUNDS=${ROUNDS:-3}

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

cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 --rpc-url "$RPC" --private-key $KEY >/dev/null
CREATE2_DEPLOYER=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3 \
  | awk '/^contractAddress/{print $2}')

log "deploying v4 and the Solidity compute hook"
(cd "$ROOT/uniswap" && CREATE2_DEPLOYER="$CREATE2_DEPLOYER" \
  forge script script/bench/DeployStableSwapBench.s.sol:DeployStableSwapBenchScript \
    --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
    --gas-estimate-multiplier 200 --broadcast --slow) >"$WORK/deploy.log" 2>&1 \
  || { tail -40 "$WORK/deploy.log"; exit 1; }

addr_of() { grep -m1 "^  $1" "$WORK/deploy.log" | grep -oE '0x[0-9a-fA-F]{40}' || true; }
STYLUS_DEPLOYER=$(addr_of "stylusDeployer")
POOL_MANAGER=$(addr_of "poolManager")
CURRENCY0=$(addr_of "currency0")
CURRENCY1=$(addr_of "currency1")
FIXTURE=$(addr_of "fixture")
SOLIDITY_HOOK=$(addr_of "StableSwap (sol)")
grep -E "^  [a-zA-Z].*0x" "$WORK/deploy.log" | sed 's/^  //'

log "mining and deploying the Rust twin"
(cd "$ROOT/stylus" && cargo stylus get-initcode --contract stylus-native-stableswap) 2>/dev/null \
  | tail -1 >"$WORK/initcode.hex"
CONSTRUCTOR=$( (cd "$ROOT/stylus" && cargo stylus constructor --contract stylus-native-stableswap) 2>/dev/null | tail -1)
(cd "$ROOT/stylus" && cargo run -q -p stylus-hook-miner -- \
  --initcode-file "$WORK/initcode.hex" --permissions before-swap \
  --constructor-signature "$CONSTRUCTOR" --constructor-args "$POOL_MANAGER" \
  --deployer "$STYLUS_DEPLOYER") >"$WORK/mine.log" 2>&1 || { cat "$WORK/mine.log"; exit 1; }
SALT=$(grep -m1 '^salt:' "$WORK/mine.log" | grep -oE '0x[0-9a-fA-F]{64}')
(cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-native-stableswap --no-verify \
  -e "$RPC" --private-key $KEY --deployer-address "$STYLUS_DEPLOYER" --deployer-salt "$SALT" \
  --constructor-args "$POOL_MANAGER") >"$WORK/native.log" 2>&1 \
  || { tail -30 "$WORK/native.log"; exit 1; }
RUST_HOOK=$(grep -aoiE '(contract deployed at address|deployed code at address)[^0]*0x[0-9a-fA-F]{40}' \
  "$WORK/native.log" | grep -oE '0x[0-9a-fA-F]{40}' | tail -1)
echo "rust hook: $RUST_HOOK"
read -r INIT INIT_CACHED <<<"$(cast call $ARB_WASM 'programInitGas(address)(uint64,uint64)' \
  "$RUST_HOOK" --rpc-url "$RPC" | awk '{print $1}' | tr '\n' ' ')"

log "opening one pool per variant"
open_pool() {
  cast send "$FIXTURE" "open(address,address,address,uint128)" \
    "$CURRENCY0" "$CURRENCY1" "$1" "$LIQUIDITY" --rpc-url "$RPC" --private-key $KEY >/dev/null
}
open_pool 0x0000000000000000000000000000000000000000
open_pool "$SOLIDITY_HOOK"
open_pool "$RUST_HOOK"

# the two curves must agree, or the benchmark compares two different AMMs
for n in 1000000000000000000 5000000000000000000; do
  a=$(cast call "$SOLIDITY_HOOK" 'getY(uint256,uint256,uint256,uint256)(uint256)' "$n" 1000000000000000000000 600000000000000000000 100 --rpc-url "$RPC" | awk '{print $1}')
  b=$(cast call "$RUST_HOOK" 'getY(uint256,uint256,uint256,uint256)(uint256)' "$n" 1000000000000000000000 600000000000000000000 100 --rpc-url "$RPC" | awk '{print $1}')
  [ "$a" = "$b" ] || { echo "the two curves disagree at amountIn=$n: $a vs $b"; exit 1; }
done
echo "both hooks price the swap identically"

swap_gas() {
  cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$1" "$SWAP_AMOUNT" true \
    --rpc-url "$RPC" --private-key $KEY | awk '/^gasUsed/{print $2}'
}

log "sweeping the amount of work"
# warm every slot first
for i in 0 1 2; do swap_gas "$i" >/dev/null; done

log "swapping through both ($ROUNDS rounds; the first is cold)"
printf '%-8s %14s %14s %14s\n' round solidity rust delta
for round in $(seq 1 "$ROUNDS"); do
  s=$(swap_gas 1); r=$(swap_gas 2)
  printf '%-8s %14s %14s %14s\n' "$round" "$s" "$r" "$((r - s))"
done

BASE=$(swap_gas 0)
echo
printf 'baseline swap with no hook at all        %10s\n' "$BASE"
printf 'the Rust hook pays this on every call:\n'
printf '  loading the WASM program, uncached      %10s\n' "$INIT"
printf '  the same once the contract is cached    %10s\n' "$INIT_CACHED"
echo
echo "note: this dev node has no CacheManager, so the cached figure comes from"
echo "      ArbWasm.programInitGas rather than a measurement."
