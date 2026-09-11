#!/usr/bin/env bash
# The pm-AMM's arithmetic, in Solidity and in Rust.
#
#   ./bench-pmamm.bash
#
# Every hook in BENCHMARK.md loses to its Solidity twin, because Stylus trades a lower marginal cost
# of computation for a fixed cost per call. TWAMM looked like the exception until the version
# actually deployed turned out to use plain integer `mulDiv` and no transcendentals at all.
#
# The pm-AMM is a better bet for one reason: its arithmetic cannot be removed. With `z = (y-x)/L`,
# Paradigm's invariant is
#
#   f(y)  = (y - x)·Phi(z) + L·phi(z) - y
#   f'(y) = Phi(z) - 1
#
# which is transcendental in y. No closed form exists, so a solve is mandatory and every iteration
# needs the Gaussian CDF and PDF. `Gnome101/Pm-AMM-Hook` — the one v4 hook that implements a pm-AMM —
# solves it by 100-step bisection, which is 200 Gaussian evaluations per swap. This measures the
# honest floor instead: the same solve by Newton.
#
# Both sides run `primitivefinance/solstat`'s algorithm. The Rust in `stylus/native-gaussian` is a
# bit-exact port of it, Solmate's `expWad` included, and both test suites pin the same table of
# values — so what is measured here is the language and not a precision trade.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-pmamm-bench}
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

log "deploying both implementations"
SOL_MATH=$( (cd "$ROOT/uniswap" && forge create src/PmAmmMath.sol:PmAmmMath \
  --rpc-url "$RPC" --private-key $KEY --broadcast) 2>&1 \
  | grep -oE 'Deployed to: 0x[0-9a-fA-F]{40}' | grep -oE '0x[0-9a-fA-F]{40}')
[ -n "$SOL_MATH" ] || { echo "the Solidity side did not deploy"; exit 1; }
echo "  solidity: $SOL_MATH"

# No address mining here: neither contract is a hook, so there are no permission bits to encode.
(cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-native-gaussian --no-verify \
  -e "$RPC" --private-key $KEY) >"$WORK/rust.log" 2>&1 || { tail -20 "$WORK/rust.log"; exit 1; }
RUST_MATH=$(grep -aoiE '(contract deployed at address|deployed code at address)[^0]*0x[0-9a-fA-F]{40}' \
  "$WORK/rust.log" | grep -oE '0x[0-9a-fA-F]{40}' | tail -1)
[ -n "$RUST_MATH" ] || { tail -20 "$WORK/rust.log"; exit 1; }
echo "  rust:     $RUST_MATH"
read -r INIT INIT_CACHED <<<"$(cast call $ARB_WASM 'programInitGas(address)(uint64,uint64)' \
  "$RUST_MATH" --rpc-url "$RPC" | awk '{print $1}' | tr '\n' ' ')"

# Cached, because that is the state any contract with users would be in: it is a one-off bid on a
# real chain, and an uncached program pays the full WASM load on every call forever.
cached=$(cache_stylus_program "$RUST_MATH" "$EOA")
echo "  rust program cached: $cached"

log "the two must agree before anything is timed"
agree=1
for x in -3000000000000000000 -1000000000000000000 -400000000000000000 0 400000000000000000 1000000000000000000 3000000000000000000; do
  for f in 'erfc(int256)(int256)' 'cdf(int256)(int256)' 'pdf(int256)(int256)'; do
    a=$(cast call "$SOL_MATH" "$f" "$x" --rpc-url "$RPC" | awk '{print $1}')
    b=$(cast call "$RUST_MATH" "$f" "$x" --rpc-url "$RPC" | awk '{print $1}')
    [ "$a" = "$b" ] || { echo "  MISMATCH ${f%%(*}($x): $a vs $b"; agree=0; }
  done
done
for n in 1 4 8; do
  a=$(cast call "$SOL_MATH" 'work(uint256)(int256)' "$n" --rpc-url "$RPC" | awk '{print $1}')
  b=$(cast call "$RUST_MATH" 'work(uint256)(int256)' "$n" --rpc-url "$RPC" | awk '{print $1}')
  [ "$a" = "$b" ] || { echo "  MISMATCH work($n): $a vs $b"; agree=0; }
  printf '  work(%s) = %s\n' "$n" "$a"
done
[ "$agree" = 1 ] || { echo "the two implementations disagree; the benchmark would be meaningless"; exit 1; }
echo "  identical to the wei on every value checked"

est() { cast estimate "$1" "$2" "${@:3}" --rpc-url "$RPC" --from "$EOA" 2>/dev/null | awk '{print $1}'; }

log "the Gaussian primitives, one at a time"
# Against a real no-op, not against `f(0)`: only `erfc` short-circuits at zero, so using the zero
# call as a baseline silently subtracts nearly the whole cost of `expWad` and `pdf`.
X=400000000000000000
SB=$(est "$SOL_MATH" 'baseline(int256)' "$X"); RB=$(est "$RUST_MATH" 'baseline(int256)' "$X")
printf '%-10s %12s %12s %8s\n' "" solidity rust ratio
for f in expWad erfc cdf pdf; do
  ds=$(( $(est "$SOL_MATH" "$f(int256)" "$X") - SB ))
  dr=$(( $(est "$RUST_MATH" "$f(int256)" "$X") - RB ))
  printf '%-10s %12s %12s %8s\n' "$f" "$ds" "$dr" \
    "$(python3 -c "print(f'{$ds/$dr:.2f}x' if $dr>0 else 'n/a')")"
done

log "the solve, by how many Newton steps it takes"
printf '%-12s %12s %12s %8s\n' iterations solidity rust ratio
s0=$(est "$SOL_MATH" 'work(uint256)' 0); r0=$(est "$RUST_MATH" 'work(uint256)' 0)
for n in 1 2 4 6 8; do
  s=$(est "$SOL_MATH" 'work(uint256)' "$n"); r=$(est "$RUST_MATH" 'work(uint256)' "$n")
  ds=$((s - s0)); dr=$((r - r0))
  printf '%-12s %12s %12s %8s\n' "$n" "$ds" "$dr" \
    "$(python3 -c "print(f'{$ds/$dr:.2f}x' if $dr>0 else 'n/a')")"
done

log "why: the same expression, by how wide its operands are"
# The EVM charges 5 gas for MUL and 5 for DIV whatever the operands are. A U256 in WASM is four
# 64-bit limbs, and ruint's cost tracks how many of them are non-zero. So Stylus should look good on
# narrow values and bad on wide ones — and `expWad` works in a 2^96 basis, where everything is wide.
mul_div() {  # label a b c
  local lo hi ds dr
  lo=$(est "$SOL_MATH" 'mulDivLoop(uint256,uint256,uint256,uint256)' 0 "$2" "$3" "$4")
  hi=$(est "$SOL_MATH" 'mulDivLoop(uint256,uint256,uint256,uint256)' 100 "$2" "$3" "$4")
  ds=$((hi - lo))
  lo=$(est "$RUST_MATH" 'mulDivLoop(uint256,uint256,uint256,uint256)' 0 "$2" "$3" "$4")
  hi=$(est "$RUST_MATH" 'mulDivLoop(uint256,uint256,uint256,uint256)' 100 "$2" "$3" "$4")
  dr=$((hi - lo))
  printf '%-34s %12s %12s %8s\n' "$1" "$ds" "$dr" \
    "$(python3 -c "print(f'{$ds/$dr:.2f}x' if $dr>0 else 'n/a')")"
}
printf '%-34s %12s %12s %8s\n' "100 iterations of a*b/c" solidity rust ratio
mul_div "operands ~2^32, one limb"   4294967296 4294967296 65536
mul_div "operands ~2^96, two limbs"  79228162514264337593543950336 79228162514264337593543950336 4294967296
mul_div "operands ~2^128, four limbs" 340282366920938463463374607431768211456 1329227995784915872903807060280344576 79228162514264337593543950336

echo
printf 'the Rust program pays this on every call, uncached   %10s\n' "$INIT"
printf '  and this once cached, which is what is measured    %10s\n' "$INIT_CACHED"
echo
echo "For the verdict, compare the saving on a solve against the cached handicap a Stylus hook"
echo "carries: 7,916 gas, measured in bench-twamm.bash against the production TWAMM hook."
