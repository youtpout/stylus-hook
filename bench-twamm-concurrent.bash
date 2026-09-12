#!/usr/bin/env bash
# What a TWAMM costs when several long-term orders are running at once.
#
#   ./bench-twamm-concurrent.bash
#
# `bench-twamm.bash` measures one order stream over four expiries, which is the quiet case. A pool
# that anyone uses is not in it. TWAMM orders are placed independently, by people who do not
# coordinate, so a real pool carries several streams ending at different times — and every one of
# them puts another occupied interval on the grid.
#
# That matters because catching a pool up is a loop over occupied intervals, and each one runs the
# closed form again. The marginal cost per interval is the whole comparison, and it is paid once per
# concurrent stream that ends between two touches of the pool. So concurrency is the axis that
# decides whether a Stylus TWAMM is worth deploying, and it is the axis neither implementation's
# own benchmarks sweep.
#
# This script sweeps it directly. For each M it places M order streams expiring on consecutive grid
# points, lets them all come due with nobody touching the pool, and then measures the single swap
# that has to catch up across all M at once — on the Rust hook and on the production Solidity hook,
# same pool manager, same grid, same order sizes.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-twamm-concurrent}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
EOA=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E
ARB_OWNER=0x0000000000000000000000000000000000000070
ARB_WASM=0x0000000000000000000000000000000000000071
LIQUIDITY=1000000000000000000000
SWAP_AMOUNT=1000000000000000000
ORDER_SIZE=${ORDER_SIZE:-1000000000000000000}
# The concurrency levels to sweep: how many order streams end between two touches of the pool.
STREAMS=${STREAMS:-"1 2 4 8 16"}
# A nitro dev node has no `evm_increaseTime`, so the only way past an expiry is to wait for it.
# Five seconds a step rather than the sixty the contract defaults to.
export EXPIRATION_INTERVAL=${EXPIRATION_INTERVAL:-5}

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
trap '[ -n "${KEEP:-}" ] || { docker rm -f "$CONTAINER" >/dev/null 2>&1 || true; rm -rf "$WORK"; }' EXIT
for _ in $(seq 1 60); do cast chain-id --rpc-url "$RPC" >/dev/null 2>&1 && break; sleep 1; done

cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 --rpc-url "$RPC" --private-key $KEY >/dev/null
# The complete TWAMM needs two code fragments, which is ArbOS 61 and up. The dev image ships 59.
ensure_arbos 60 && ensure_arbos 61
CREATE2_DEPLOYER=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3 \
  | awk '/^contractAddress/{print $2}')

log "deploying v4 and the Solidity side"
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
export POOL_MANAGER="$POOL_MANAGER"

log "mining and deploying the Rust hook"
RUST_HOOK=$(deploy_fragmented_hook stylus-native-twamm \
  before-initialize,before-add-liquidity,before-swap \
  'stylus_constructor(address)' "$POOL_MANAGER" "$STYLUS_DEPLOYER" "$WORK")
[ -n "$RUST_HOOK" ] || { echo "could not deploy the Rust hook"; exit 1; }
echo "  rust hook: $RUST_HOOK"
read -r INIT INIT_CACHED <<<"$(cast call $ARB_WASM 'programInitGas(address)(uint64,uint64)' \
  "$RUST_HOOK" --rpc-url "$RPC" | awk '{print $1}' | tr '\n' ' ')"

GRID=$(cast call "$RUST_HOOK" 'expirationInterval()(uint256)' --rpc-url "$RPC" | awk '{print $1}')

log "deploying the production TWAMM hook as the control"
# akshatmittal/v4-twamm-hook: Uniswap Labs and Zaha Studio, audited by ABDK and Certora, live on
# Base and Unichain. A different algorithm to the Rust hook, deliberately — that is what production
# does — so this measures two implementations rather than two compilers.
PROD_HOOK=$(deploy_production_twamm "$POOL_MANAGER" "$GRID" "$EOA" "$CREATE2_DEPLOYER" "$WORK")
[ -n "$PROD_HOOK" ] || { echo "could not deploy the production hook"; exit 1; }
echo "  prod hook: $PROD_HOOK  (interval ${GRID}s, same grid)"

log "opening one pool per variant"
open_pool() {
  cast send "$FIXTURE" "open(address,address,address,uint128)" \
    "$CURRENCY0" "$CURRENCY1" "$1" "$LIQUIDITY" --rpc-url "$RPC" --private-key $KEY >/dev/null
}
# The pool key is (currency0, currency1, 3000, 60, hooks), so one pool per hook and no repeats.
open_pool 0x0000000000000000000000000000000000000000   # pool 0, no hook
open_pool "$RUST_HOOK"                                  # pool 1
open_pool "$PROD_HOOK"                                  # pool 2

swap_gas() {
  local out
  if ! out=$(cast send "$FIXTURE" "swap(uint256,uint256,bool)" "$1" "$SWAP_AMOUNT" true \
      --rpc-url "$RPC" --private-key $KEY 2>&1); then
    echo "swap through pool $1 reverted:" >&2
    printf '%s\n' "$out" | head -3 >&2
    return 1
  fi
  printf '%s\n' "$out" | awk '/^gasUsed/{print $2}'
}

# A nitro dev node only makes a block when it has a transaction to put in one, so its clock does not
# move on its own. Poking it with a no-op is what carries `block.timestamp` past an expiry.
wait_past() {
  local target=$1
  while [ "$(cast block latest -f timestamp --rpc-url "$RPC")" -le "$target" ]; do
    sleep 1
    cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 \
      --rpc-url "$RPC" --private-key $KEY >/dev/null
  done
}

submit_rust() {  # zeroForOne, absolute expiration
  cast send "$FIXTURE" "submitTwammOrder(uint256,address,bool,uint256,uint256)" \
    1 "$RUST_HOOK" "$1" "$2" "$ORDER_SIZE" --rpc-url "$RPC" --private-key $KEY >/dev/null
}
# The production hook takes a duration and rounds it onto its own grid, so the duration has to be
# recomputed against the clock at the moment the order is mined.
submit_prod() {  # zeroForOne, absolute expiration
  local t it dur
  t=$(cast block latest -f timestamp --rpc-url "$RPC")
  it=$(( t / GRID * GRID ))
  dur=$(( $2 - it ))
  cast send "$FIXTURE" "submitProductionTwammOrder(uint256,address,bool,uint256,uint256)" \
    2 "$PROD_HOOK" "$1" "$dur" "$ORDER_SIZE" --rpc-url "$RPC" --private-key $KEY >/dev/null
}

log "warming every pool and caching the Stylus program"
for i in 0 1 2; do swap_gas "$i" >/dev/null; done
echo "  program cached: $(cache_stylus_program "$RUST_HOOK" "$EOA")"
for i in 0 1 2; do swap_gas "$i" >/dev/null; done

BASE=$(swap_gas 0)
RUST_IDLE=$(swap_gas 1)
PROD_IDLE=$(swap_gas 2)
printf '  %-46s %10s\n' "baseline swap, no hook" "$BASE"
printf '  %-46s %10s\n' "Rust hook, pool idle" "$RUST_IDLE"
printf '  %-46s %10s\n' "production Solidity hook, pool idle" "$PROD_IDLE"

log "sweeping how many order streams are running at once"
echo "Each row places M streams ending on consecutive grid points, lets them all come due with"
echo "nobody touching the pool, and measures the one swap that catches up across all M."
echo
printf '%8s %12s %12s %14s %14s %10s\n' \
  streams "rust swap" "sol swap" "rust/stream" "sol/stream" "saving"

for M in $STREAMS; do
  # Bring both hooks up to date, so this batch starts from a clean clock.
  swap_gas 1 >/dev/null; swap_gas 2 >/dev/null

  now=$(cast block latest -f timestamp --rpc-url "$RPC")
  # This batch sends 4M transactions before the first expiry may come due, and each one is a round
  # trip to the node. Under-budget that lead and the orders land in the past, which the hooks reject
  # rather than silently mis-price — so the lead is generous and checked afterwards.
  lead=$(( 40 + 10 * M ))
  base=$(( (now + lead) / GRID * GRID ))
  last=$(( base + M * GRID ))

  for k in $(seq 1 "$M"); do
    e=$(( base + k * GRID ))
    submit_rust true "$e";  submit_rust false "$e"
    submit_prod true "$e";  submit_prod false "$e"
  done

  after=$(cast block latest -f timestamp --rpc-url "$RPC")
  if [ "$after" -ge "$base" ]; then
    echo "submitting $((4 * M)) orders took longer than the ${lead}s lead; raise it and re-run" >&2
    exit 1
  fi

  wait_past "$last"
  rust=$(swap_gas 1)
  prod=$(swap_gas 2)

  rust_per=$(( (rust - RUST_IDLE) / M ))
  prod_per=$(( (prod - PROD_IDLE) / M ))
  printf '%8s %12s %12s %14s %14s %10s\n' \
    "$M" "$rust" "$prod" "$rust_per" "$prod_per" "$(( prod - rust ))"
done

echo
printf 'the Rust hook pays this on every call, uncached      %10s\n' "$INIT"
printf '  and this once cached, which is what is measured    %10s\n' "$INIT_CACHED"
echo
echo "The entry fee is paid once per swap however many streams are running, so it is amortised"
echo "across them: the more concurrent orders a pool carries, the smaller a share of the swap it is."
echo "That is the case for a Stylus TWAMM, and it is the case a single-stream benchmark cannot show."
