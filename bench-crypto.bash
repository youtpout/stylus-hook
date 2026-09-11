#!/usr/bin/env bash
# The two primitives a post-quantum signature needs, in Solidity and in Rust.
#
#   ./bench-crypto.bash
#
# Every other benchmark here asks how much cheaper Stylus is. This one asks whether Solidity can do
# the thing at all, because the two primitives ML-DSA (Dilithium) verification spends its gas on are
# the two the EVM has no answer for.
#
# SHAKE256 is the structural one. Stylus has `native_keccak256` and the EVM has the `KECCAK256`
# opcode, and both are Keccak-256 with the `0x01` pad compiled in. SHAKE pads with `0x1f` and squeezes
# an arbitrary length, so neither built-in can produce it -- both languages have to run
# Keccak-f[1600] themselves. That is 24 rounds of 64-bit rotations over 25 lanes: one WASM
# instruction per rotation, four EVM opcodes and a mask.
#
# The NTT is a word-size one. ML-DSA's modulus is 8380417, twenty-three bits. The EVM has one word
# and it is 256 bits wide, so every butterfly pays a full `MULMOD`.
#
# Both sides are held to the standard before anything is timed: NIST's SHAKE256 vectors, XKCP's
# vector for the permutation on a zero state, and the same transform output. And both sides are
# written the way each language would really write them -- the Solidity permutation and butterfly are
# unrolled assembly, because the readable versions spent most of their gas on bounds checks and
# comparing against that would be comparing against a strawman. `keccakFLoopChecked` and
# `nttLoopChecked` are kept so the cost of the checks is visible rather than assumed.
#
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-crypto-bench}
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
SOL=$( (cd "$ROOT/uniswap" && forge create src/CryptoBench.sol:CryptoBench \
  --rpc-url "$RPC" --private-key $KEY --broadcast) 2>&1 \
  | grep -oE 'Deployed to: 0x[0-9a-fA-F]{40}' | grep -oE '0x[0-9a-fA-F]{40}')
[ -n "$SOL" ] || { echo "the Solidity side did not deploy"; exit 1; }
echo "  solidity: $SOL"

(cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-native-crypto --no-verify \
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


# `cast` annotates large integers ("17376452488221285863 [1.737e19]"); drop the annotation.
call() {
  cast call "$1" "$2" "${@:3}" --rpc-url "$RPC" 2>&1 \
    | sed -E 's/ \[[0-9.e+-]+\]//g' | tr '\n' ' ' | xargs
}
est() { cast estimate "$1" "$2" "${@:3}" --rpc-url "$RPC" --from "$EOA" 2>/dev/null | awk '{print $1}'; }

log "both sides must match the standard before anything is timed"
agree=1
pin() {  # label, address, signature, args..., expected
  local want="${*: -1}" got
  got=$(call "$2" "$3" "${@:4:$#-4}") || got="<the call reverted>"
  if [ "$got" != "$want" ]; then echo "  MISMATCH $1: $got"; echo "         attendu: $want"; agree=0
  else echo "  ok  $1"; fi
}
# XKCP's vector for Keccak-f[1600] on an all-zero state, as lane 0.
XKCP=17376452488221285863  # 0xf1258f7940e1dde7
for pair in "solidity:$SOL" "rust:$RUST"; do
  pin "${pair%%:*} keccak-f(0)" "${pair#*:}" 'keccakFLoop(uint256)(uint256)' 1 "$XKCP"
done
# NIST's SHAKE256 vectors.
for pair in "solidity:$SOL" "rust:$RUST"; do
  a=${pair#*:}
  pin "${pair%%:*} shake256('')" "$a" 'shake256(bytes)(bytes32)' 0x \
    0x46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762f
  pin "${pair%%:*} shake256('abc')" "$a" 'shake256(bytes)(bytes32)' 0x616263 \
    0x483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739
done
# And the transform, which both sides have to compute identically.
sv=$(call "$SOL" 'nttVector()(uint256[])') || sv="<solidity reverted>"
rv=$(call "$RUST" 'nttVector()(uint256[])') || rv="<rust reverted>"
if [ "$sv" = "$rv" ]; then echo "  ok  ntt(0..255)[0..4] = $sv"
else echo "  MISMATCH ntt: $sv vs $rv"; agree=0; fi
[ "$agree" = 1 ] || { echo "the two implementations disagree; the benchmark would be meaningless"; exit 1; }

log "Keccak-f[1600], the permutation SHAKE is built from"
SB=$(est "$SOL" 'baseline(uint256)' 0); RB=$(est "$RUST" 'baseline(uint256)' 0)
row() {  # label, solidity, rust
  printf '%-38s %12s %12s %9s\n' "$1" "$2" "$3" \
    "$(python3 -c "print(f'{$2/$3:.2f}x' if $3>0 else 'n/a')")"
}
marginal() {  # label, signature, low, high, args...
  local sl sh rl rh d
  sl=$(est "$SOL" "$2" "$3" "${@:5}");  sh=$(est "$SOL" "$2" "$4" "${@:5}")
  rl=$(est "$RUST" "$2" "$3" "${@:5}"); rh=$(est "$RUST" "$2" "$4" "${@:5}")
  d=$(( $4 - $3 ))
  row "$1" "$(( (sh - sl) / d ))" "$(( (rh - rl) / d ))"
}
printf '%-38s %12s %12s %9s\n' "" solidity rust ratio
marginal "one permutation" 'keccakFLoop(uint256)' 0 20
# What Solidity's bounds checks cost, so the control above is visibly not a strawman.
s_checked=$(( ( $(est "$SOL" 'keccakFLoopChecked(uint256)' 20) - $(est "$SOL" 'keccakFLoopChecked(uint256)' 0) ) / 20 ))
s_asm=$(( ( $(est "$SOL" 'keccakFLoop(uint256)' 20) - $(est "$SOL" 'keccakFLoop(uint256)' 0) ) / 20 ))
printf '%-38s %12s %12s %9s\n' "  (same, bounds-checked arrays)" "$s_checked" "—" \
  "$(python3 -c "print(f'{$s_checked/$s_asm:.2f}x the assembly')")"

log "the SDK's keccak, as the control the numbers are read against"
# `crypto::keccak` goes to the host; `keccak256` is an opcode. Neither can produce SHAKE, but the gap
# between them and the permutation says how much of the cost is the permutation rather than plumbing.
sl=$(est "$SOL" 'keccakOpcodeLoop(uint256,bytes32)' 0 0x0000000000000000000000000000000000000000000000000000000000000001)
sh=$(est "$SOL" 'keccakOpcodeLoop(uint256,bytes32)' 200 0x0000000000000000000000000000000000000000000000000000000000000001)
rl=$(est "$RUST" 'keccakSdkLoop(uint256,bytes32)' 0 0x0000000000000000000000000000000000000000000000000000000000000001)
rh=$(est "$RUST" 'keccakSdkLoop(uint256,bytes32)' 200 0x0000000000000000000000000000000000000000000000000000000000000001)
printf '%-38s %12s %12s %9s\n' "" solidity rust ratio
row "one 32-byte hash, built in" "$(( (sh - sl) / 200 ))" "$(( (rh - rl) / 200 ))"

log "SHAKE256, at the shapes ML-DSA actually uses"
printf '%-38s %12s %12s %9s\n' "" solidity rust ratio
marginal "absorb 32 B, squeeze 32 B"    'shake256Loop(uint256,uint256,uint256)' 0 8 32 32
marginal "absorb 32 B, squeeze 168 B"   'shake256Loop(uint256,uint256,uint256)' 0 8 32 168
marginal "absorb 200 B, squeeze 1088 B" 'shake256Loop(uint256,uint256,uint256)' 0 4 200 1088

log "the number-theoretic transform over ML-DSA's 23-bit prime"
printf '%-38s %12s %12s %9s\n' "" solidity rust ratio
marginal "one forward NTT, 256 coefficients" 'nttLoop(uint256)' 0 8
n_checked=$(( ( $(est "$SOL" 'nttLoopChecked(uint256)' 8) - $(est "$SOL" 'nttLoopChecked(uint256)' 0) ) / 8 ))
n_asm=$(( ( $(est "$SOL" 'nttLoop(uint256)' 8) - $(est "$SOL" 'nttLoop(uint256)' 0) ) / 8 ))
printf '%-38s %12s %12s %9s\n' "  (same, bounds-checked arrays)" "$n_checked" "—" \
  "$(python3 -c "print(f'{$n_checked/$n_asm:.2f}x the assembly')")"
# Montgomery has no Solidity counterpart: MULMOD already reduces for free.
r_mont=$(( ( $(est "$RUST" 'nttMontgomeryLoop(uint256)' 8) - $(est "$RUST" 'nttMontgomeryLoop(uint256)' 0) ) / 8 ))
printf '%-38s %12s %12s %9s\n' "  rust in the Montgomery domain" "—" "$r_mont" \
  "$(python3 -c "print(f'{$n_asm/$r_mont:.2f}x solidity')")"

log "what that projects to for one ML-DSA-44 verification"
# The shape of the algorithm, not a measurement of it: ExpandA rejection-samples a 4x4 matrix of
# polynomials out of SHAKE128, which is where nearly all the permutations go, plus the challenge and
# the message hash. Then 4 forward NTTs, 16 pointwise products and 4 inverse ones.
PERMS=90
NTTS=9
s_perm=$s_asm
r_perm=$(( ( $(est "$RUST" 'keccakFLoop(uint256)' 20) - $(est "$RUST" 'keccakFLoop(uint256)' 0) ) / 20 ))
r_ntt=$(( ( $(est "$RUST" 'nttLoop(uint256)' 8) - $(est "$RUST" 'nttLoop(uint256)' 0) ) / 8 ))
printf '  %s permutations and %s transforms, at the rates above:\n\n' "$PERMS" "$NTTS"
printf '%-38s %12s %12s %9s\n' "" solidity rust ratio
row "permutations" "$((PERMS * s_perm))" "$((PERMS * r_perm))"
row "transforms" "$((NTTS * n_asm))" "$((NTTS * r_ntt))"
row "total" "$((PERMS * s_perm + NTTS * n_asm))" "$((PERMS * r_perm + NTTS * r_ntt))"
echo
echo "  This is an estimate from the algorithm's structure, not a measured verification."

echo
printf 'the Rust program pays this on every call, uncached   %10s\n' "$INIT"
printf '  and this once cached, which is what is measured    %10s\n' "$INIT_CACHED"
echo
echo "A swap on Arbitrum costs about 115,000 gas with no hook. Read the total above against that."
