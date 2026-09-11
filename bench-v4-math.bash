#!/usr/bin/env bash
# v4-core's swap math, in Solidity and in Rust.
#
#   ./bench-v4-math.bash
#
# The control side is not a reimplementation: `uniswap/src/V4MathBench.sol` calls `SwapMath`,
# `TickMath`, `SqrtPriceMath`, `FullMath` and `TickBitmap` straight out of v4-core. The Rust side is
# `stylus/v4-math`, a port tested against v4-core's own vectors -- every unit test in
# `test/libraries/` for those libraries, same inputs, same expected values. So both sides compute
# identical numbers by identical steps, and what is measured is the language.
#
# The headline is `walkSwap`: `Pool.swap`'s loop without the storage. Find the next initialised tick,
# price the step up to it, cross, repeat. Replaying that loop is what OpenZeppelin's
# `AntiSandwichHook` and Uniswap's own `alf/SwapSimulator` do on every swap -- pure arithmetic with a
# swap's worth of it, which is the one shape of hook where Stylus can win.
#
# As in bench-pmamm.bash the Rust is built at `opt-level = 3`, which needs
# `--llvm-memory-copy-fill-lowering` in `Stylus.toml` or ArbOS refuses to activate the contract. It
# is worth about 2x the gas. The Solidity is built exactly as every other measurement in
# BENCHMARK.md: optimizer on, 200 runs, `via_ir = false`.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-v4math-bench}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
EOA=0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E
ARB_OWNER=0x0000000000000000000000000000000000000070
ARB_WASM=0x0000000000000000000000000000000000000071

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

# L1 pricing off, so what is reported is L2 execution only.
cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 --rpc-url "$RPC" --private-key $KEY >/dev/null

# At `opt-level = 3` the Rust side is 34 KB, which is two fragments, and fragments need ArbOS 61.
# Published dev images boot 59, so step it up -- otherwise activation reverts with no data at all.
ensure_arbos 60 || true
ensure_arbos 61 || true
frags=$(cast call 0x000000000000000000000000000000000000006b \
  "getMaxStylusContractFragments()(uint16)" --rpc-url "$RPC" 2>/dev/null | awk '{print $1}')
echo "  max stylus contract fragments: ${frags:-unavailable}"

log "deploying both implementations"
SOL=$( (cd "$ROOT/uniswap" && forge create src/V4MathBench.sol:V4MathBench \
  --rpc-url "$RPC" --private-key $KEY --broadcast) 2>&1 \
  | grep -oE 'Deployed to: 0x[0-9a-fA-F]{40}' | grep -oE '0x[0-9a-fA-F]{40}')
[ -n "$SOL" ] || { echo "the Solidity side did not deploy"; exit 1; }
echo "  solidity: $SOL"

(cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-native-v4-math --no-verify \
  -e "$RPC" --private-key $KEY) >"$WORK/rust.log" 2>&1 || { tail -20 "$WORK/rust.log"; exit 1; }
RUST=$(grep -aoiE '(contract deployed at address|deployed code at address)[^0]*0x[0-9a-fA-F]{40}' \
  "$WORK/rust.log" | grep -oE '0x[0-9a-fA-F]{40}' | tail -1)
[ -n "$RUST" ] || { tail -20 "$WORK/rust.log"; exit 1; }
echo "  rust:     $RUST"
read -r INIT INIT_CACHED <<<"$(cast call $ARB_WASM 'programInitGas(address)(uint64,uint64)' \
  "$RUST" --rpc-url "$RPC" | awk '{print $1}' | tr '\n' ' ')"

# Cached, because that is the state any contract with users would be in: a one-off bid on a real
# chain, where an uncached program pays the full WASM load on every call forever.
cached=$(cache_stylus_program "$RUST" "$EOA")
echo "  rust program cached: $cached"

P11=79228162514264337593543950336              # sqrt price at tick 0
P101=79623317895830914510639640423             # 101/100
MIN_P=4295128739
MAX_P=1461446703485210103287273052203988822378723970342

call() { cast call "$1" "$2" "${@:3}" --rpc-url "$RPC" 2>&1 | tr '\n' ' ' | xargs; }
est() { cast estimate "$1" "$2" "${@:3}" --rpc-url "$RPC" --from "$EOA" 2>/dev/null | awk '{print $1}'; }

log "the two must agree before anything is timed"
agree=1
check() {  # signature, args...
  local a b
  a=$(call "$SOL" "$@"); b=$(call "$RUST" "$@")
  if [ "$a" != "$b" ]; then echo "  MISMATCH ${1%%(*}($*): $a vs $b"; agree=0; fi
}
for t in -887272 -887271 -100000 -60 -1 0 1 60 100000 887271 887272; do
  check 'getSqrtPriceAtTick(int24)(uint160)' "$t"
done
for p in $MIN_P 4295343490 1000000000000000000000 $P11 $P101 \
         1461373636630004318706518188784493106690254656249; do
  check 'getTickAtSqrtPrice(uint160)(int24)' "$p"
done
check 'mulDiv(uint256,uint256,uint256)(uint256)' \
  115792089237316195423570985008687907853269984665640564039457584007913129639935 \
  115792089237316195423570985008687907853269984665640564039457584007913129639934 \
  115792089237316195423570985008687907853269984665640564039457584007913129639935
for amt in -1000000000000000000 1000000000000000000 -10 4; do
  check 'computeSwapStep(uint160,uint160,uint128,int256,uint24)(uint160,uint256,uint256,uint256)' \
    $P11 $P101 2000000000000000000 "$amt" 600
done
# And the workload itself, which is the only equality that really matters.
for steps in 1 4 16 64; do
  for amt in -1000000000000000000000 1000000000000000000000; do
    check 'walkSwap(uint160,uint160,int24,uint128,int256,uint24,uint256)(uint160,uint256,uint256,uint256)' \
      $P11 $((MIN_P + 1)) 60 1000000000000000000 "$amt" 3000 "$steps"
  done
done
[ "$agree" = 1 ] || { echo "the two implementations disagree; the benchmark would be meaningless"; exit 1; }
echo "  identical on every tick, price, step and walk checked"

log "the workload: Pool.swap's loop, by how many ticks the swap crosses"
# A 1e18-liquidity pool taking 1000 ETH of exact input, 60 tick spacing: the price runs, so the step
# count is the dial. This is the shape of a swap that a replaying hook has to redo.
printf '%-10s %12s %12s %10s %10s\n' "ticks" solidity rust ratio "saving"
s0=$(est "$SOL" 'walkSwap(uint160,uint160,int24,uint128,int256,uint24,uint256)' \
  $P11 $((MIN_P + 1)) 60 1000000000000000000 -1000000000000000000000 3000 0)
r0=$(est "$RUST" 'walkSwap(uint160,uint160,int24,uint128,int256,uint24,uint256)' \
  $P11 $((MIN_P + 1)) 60 1000000000000000000 -1000000000000000000000 3000 0)
printf '%-10s %12s %12s %10s %10s\n' "0 (call)" "$s0" "$r0" "" "$((s0 - r0))"
for n in 1 2 4 8 16 32 64; do
  s=$(est "$SOL" 'walkSwap(uint160,uint160,int24,uint128,int256,uint24,uint256)' \
    $P11 $((MIN_P + 1)) 60 1000000000000000000 -1000000000000000000000 3000 "$n")
  r=$(est "$RUST" 'walkSwap(uint160,uint160,int24,uint128,int256,uint24,uint256)' \
    $P11 $((MIN_P + 1)) 60 1000000000000000000 -1000000000000000000000 3000 "$n")
  printf '%-10s %12s %12s %10s %10s\n' "$n" "$s" "$r" \
    "$(python3 -c "print(f'{($s-$s0)/($r-$r0):.2f}x' if $r>$r0 else 'n/a')")" "$((s - r))"
done

log "routing: how much search a swap's gas budget buys"
# Splitting a swap across pools is computed off-chain today and handed to the chain on trust. Each
# round of the greedy walks every pool once to price one more chunk, so the work is `rounds x pools`
# replays -- and how many rounds you can afford is how good the split gets.
#
# 100 ETH across three pools at 1:1, 101:100 and 99:100 with depths 1000e18, 2000e18 and 500e18.
# `maxSteps` is 16 because that is where the walks stop growing for this trade; capping lower
# truncates them and measures a cheaper, wrong swap.
P101=79623317895830914510639640423
P99=78831026366734652303669917531
PRICES="[$P11,$P101,$P99]"
LIQ="[1000000000000000000000,2000000000000000000000,500000000000000000000]"
AMT=100000000000000000000
ROUTE='routeExactIn(uint160[],uint128[],int24,uint256,uint24,uint256,uint256)'
ROUTE_OUT='routeExactIn(uint160[],uint128[],int24,uint256,uint24,uint256,uint256)(uint256,uint256[])'

# What one pool alone returns, as the baseline the split has to beat.
single=$(call "$SOL" "$ROUTE_OUT" "$PRICES" "$LIQ" 60 $AMT 3000 1 16 | awk '{print $1}')
printf '%-22s %11s %11s %7s %11s %12s\n' "replays" solidity rust ratio "saving" "output gain"
for n in 1 2 4 8 16; do
  s=$(est "$SOL" "$ROUTE" "$PRICES" "$LIQ" 60 $AMT 3000 "$n" 16)
  r=$(est "$RUST" "$ROUTE" "$PRICES" "$LIQ" 60 $AMT 3000 "$n" 16)
  out=$(call "$RUST" "$ROUTE_OUT" "$PRICES" "$LIQ" 60 $AMT 3000 "$n" 16 | awk '{print $1}')
  printf '%-22s %11s %11s %7s %11s %12s\n' "$((n * 3)) ($n x 3 pools)" "$s" "$r" \
    "$(python3 -c "print(f'{$s/$r:.2f}x')")" "$((s - r))" \
    "$(python3 -c "print('+%d bps' % (($out - $single) * 10000 // $single))")"
done
echo
echo "  The gain dwarfs the gas on an L2 either way, so this is not about whether to search -- it is"
echo "  about how deep. A swap with no hook costs about 115,000 gas; read both columns against that"
echo "  to see how many rounds each language fits inside one swap's budget."

log "the primitives, one at a time"
SB=$(est "$SOL" 'baseline(uint256)' 0); RB=$(est "$RUST" 'baseline(uint256)' 0)
printf '%-34s %12s %12s %8s\n' "" solidity rust ratio
row() {  # label, solidity gas, rust gas
  printf '%-34s %12s %12s %8s\n' "$1" "$2" "$3" \
    "$(python3 -c "print(f'{$2/$3:.2f}x' if $3>0 else 'n/a')")"
}
row "getSqrtPriceAtTick(200000)" \
  "$(( $(est "$SOL" 'getSqrtPriceAtTick(int24)' 200000) - SB ))" \
  "$(( $(est "$RUST" 'getSqrtPriceAtTick(int24)' 200000) - RB ))"
row "getTickAtSqrtPrice(~1e21)" \
  "$(( $(est "$SOL" 'getTickAtSqrtPrice(uint160)' 1000000000000000000000) - SB ))" \
  "$(( $(est "$RUST" 'getTickAtSqrtPrice(uint160)' 1000000000000000000000) - RB ))"
row "computeSwapStep, exact in" \
  "$(( $(est "$SOL" 'computeSwapStep(uint160,uint160,uint128,int256,uint24)' $P11 $P101 2000000000000000000 -1000000000000000000 600) - SB ))" \
  "$(( $(est "$RUST" 'computeSwapStep(uint160,uint160,uint128,int256,uint24)' $P11 $P101 2000000000000000000 -1000000000000000000 600) - RB ))"
row "mulDiv, max inputs" \
  "$(( $(est "$SOL" 'mulDiv(uint256,uint256,uint256)' \
    115792089237316195423570985008687907853269984665640564039457584007913129639935 \
    115792089237316195423570985008687907853269984665640564039457584007913129639934 \
    115792089237316195423570985008687907853269984665640564039457584007913129639935) - SB ))" \
  "$(( $(est "$RUST" 'mulDiv(uint256,uint256,uint256)' \
    115792089237316195423570985008687907853269984665640564039457584007913129639935 \
    115792089237316195423570985008687907853269984665640564039457584007913129639934 \
    115792089237316195423570985008687907853269984665640564039457584007913129639935) - RB ))"

log "the marginal cost, 100 iterations deep, so the call overhead cancels"
printf '%-34s %12s %12s %8s\n' "100 iterations of" solidity rust ratio
loop() {  # label, signature, args after n
  local sl sh rl rh
  sl=$(est "$SOL" "$2" 0 "${@:3}");  sh=$(est "$SOL" "$2" 100 "${@:3}")
  rl=$(est "$RUST" "$2" 0 "${@:3}"); rh=$(est "$RUST" "$2" 100 "${@:3}")
  row "$1" "$((sh - sl))" "$((rh - rl))"
}
loop "getSqrtPriceAtTick" 'sqrtPriceAtTickLoop(uint256,int24)' 200000
loop "getTickAtSqrtPrice" 'tickAtSqrtPriceLoop(uint256,uint160)' 1000000000000000000000
loop "computeSwapStep" 'swapStepLoop(uint256,uint160,uint160,uint128,int256,uint24)' \
  $P11 $P101 2000000000000000000 -1000000000000000000 600

echo
printf 'the Rust program pays this on every call, uncached   %10s\n' "$INIT"
printf '  and this once cached, which is what is measured    %10s\n' "$INIT_CACHED"
echo
echo "The saving column is end to end, cached init gas included, so it needs no adjustment: a"
echo "positive number is a net win. The crossover sits around six ticks crossed."
