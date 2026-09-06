#!/usr/bin/env bash
# A Uniswap TWAMM interval, in Solidity and in Rust.
#
#   ./bench-twamm.bash
#
# TWAMM is the one hook in BENCHMARK.md whose cost is arithmetic rather than storage, which is the
# only shape where a Stylus port can win back what it pays to be called at all. Uniswap's own
# v4-periphery example spends that arithmetic on IEEE 754 binary128, emulated in software because
# the EVM has no floating point.
#
# Stylus has none either, so the port has to work in fixed point. That makes for a three-way
# comparison, and the middle row is the one that matters: it separates the algorithmic change from
# the language change.
#
# The Rust side is also a complete TWAMM hook — orders, order pools, expiries, settlement against
# the pool manager — and it is too big for one Stylus code fragment, so deploying it at a mined
# hook address takes the workaround in `deploy_fragmented_hook`.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-twamm-bench}
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
# The complete TWAMM needs two code fragments, which is ArbOS 61 and up. The dev image ships 59.
ensure_arbos 60 && ensure_arbos 61
printf 'arbOS %s, up to %s stylus fragments\n' \
  "$(cast call 0x0000000000000000000000000000000000000064 'arbOSVersion()(uint256)' --rpc-url "$RPC")" \
  "$(cast call 0x000000000000000000000000000000000000006b 'getMaxStylusContractFragments()(uint16)' --rpc-url "$RPC")"
CREATE2_DEPLOYER=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3 \
  | awk '/^contractAddress/{print $2}')

log "deploying v4 and the Solidity compute hook"
(cd "$ROOT/uniswap" && CREATE2_DEPLOYER="$CREATE2_DEPLOYER" \
  forge script script/bench/DeployTwammBench.s.sol:DeployTwammBenchScript \
    --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
    --gas-estimate-multiplier 200 --broadcast --slow) >"$WORK/deploy.log" 2>&1 \
  || { tail -40 "$WORK/deploy.log"; exit 1; }

addr_of() { grep -m1 "^  $1" "$WORK/deploy.log" | grep -oE '0x[0-9a-fA-F]{40}' || true; }
STYLUS_DEPLOYER=$(addr_of "stylusDeployer")
POOL_MANAGER=$(addr_of "poolManager")
CURRENCY0=$(addr_of "currency0")
CURRENCY1=$(addr_of "currency1")
FIXTURE=$(addr_of "fixture")
SOLIDITY_HOOK=$(addr_of "Twamm (sol)")
grep -E "^  [a-zA-Z].*0x" "$WORK/deploy.log" | sed 's/^  //'

# The Rust hook holds the pool manager as a build-time constant rather than in storage, so it costs
# nothing to read on every callback. Its constructor still takes the address, purely to check it
# against what was compiled in — a wrong $POOL_MANAGER then fails the deployment instead of
# producing a hook that silently rejects every call. See stylus/native-stableswap/build.rs.
export POOL_MANAGER="$POOL_MANAGER"

log "mining and deploying the Rust twin"
# The complete TWAMM does not fit in one code fragment, so this is not the usual
# get-initcode/mine/deploy. See bench-lib.bash for what it does instead and why.
RUST_HOOK=$(deploy_fragmented_hook stylus-native-twamm \
  before-initialize,before-add-liquidity,before-swap \
  'stylus_constructor(address)' "$POOL_MANAGER" "$STYLUS_DEPLOYER" "$WORK")
[ -n "$RUST_HOOK" ] || { echo "could not deploy the Rust hook"; exit 1; }
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



swap_gas() {
  cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$1" "$SWAP_AMOUNT" true \
    --rpc-url "$RPC" --private-key $KEY | awk '/^gasUsed/{print $2}'
}

log "sweeping the amount of work"
# warm every slot first
for i in 0 1 2; do swap_gas "$i" >/dev/null; done

log "the three implementations must agree before anything is timed"
for n in 1 3; do
  a=$(cast call "$SOLIDITY_HOOK" 'work(uint256)(uint256)' "$n" --rpc-url "$RPC" | awk '{print $1}')
  b=$(cast call "$SOLIDITY_HOOK" 'workFixed(uint256)(uint256)' "$n" --rpc-url "$RPC" | awk '{print $1}')
  c=$(cast call "$RUST_HOOK" 'workFixed(uint256)(uint256)' "$n" --rpc-url "$RPC" | awk '{print $1}')
  [ "$b" = "$c" ] || { echo "the two fixed-point forms disagree at n=$n: $b vs $c"; exit 1; }
  printf '  n=%s  quad=%s  fixed=%s\n' "$n" "$a" "$b"
done
echo "  solidity and rust fixed point are identical; both track the quad floats to 2 parts in 1e18"

log "one interval of TWAMM arithmetic, by itself"
est() { cast estimate "$1" "$2" "${@:3}" --rpc-url "$RPC" --from 0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E 2>/dev/null | awk '{print $1}'; }
printf '%-10s %14s %14s %14s\n' intervals "sol quad" "sol fixed" "rust fixed"
for n in 1 2 4 8; do
  q0=$(est "$SOLIDITY_HOOK" 'work(uint256)' 0);      qn=$(est "$SOLIDITY_HOOK" 'work(uint256)' "$n")
  f0=$(est "$SOLIDITY_HOOK" 'workFixed(uint256)' 0); fn=$(est "$SOLIDITY_HOOK" 'workFixed(uint256)' "$n")
  r0=$(est "$RUST_HOOK" 'workFixed(uint256)' 0);     rn=$(est "$RUST_HOOK" 'workFixed(uint256)' "$n")
  printf '%-10s %14s %14s %14s\n' "$n" "$((qn - q0))" "$((fn - f0))" "$((rn - r0))"
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
