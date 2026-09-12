#!/usr/bin/env bash
# Shared setup for the benchmark and proof scripts. Source it, do not run it.

BINARYEN_VERSION=131

# `stylus/native-counter/Stylus.toml` pins a Binaryen version so that the optimisation applied at
# deploy time is the same one `cargo stylus verify` will replay. cargo-stylus refuses to build if the
# wasm-opt on PATH is a different version, so fetch the pinned one when it is missing.
ensure_binaryen() {
  local dir="$1"
  if command -v wasm-opt >/dev/null 2>&1 &&
     wasm-opt --version 2>/dev/null | grep -q "version $BINARYEN_VERSION\b"; then
    return 0
  fi
  # The release publishes one tarball per platform, and picking the wrong one fails late and
  # obscurely: the download and the untar both succeed, and the first `cargo stylus` invocation dies
  # with "Exec format error" from inside a deploy step.
  local os arch
  case "$(uname -s)" in
    Darwin) os=macos ;;
    Linux)  os=linux ;;
    *) echo "no binaryen build for $(uname -s); install wasm-opt $BINARYEN_VERSION yourself" >&2
       return 1 ;;
  esac
  case "$(uname -m)" in
    arm64|aarch64) [ "$os" = macos ] && arch=arm64 || arch=aarch64 ;;
    x86_64|amd64)  arch=x86_64 ;;
    *) echo "no binaryen build for $(uname -m); install wasm-opt $BINARYEN_VERSION yourself" >&2
       return 1 ;;
  esac
  local url="https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-${arch}-${os}.tar.gz"
  echo "fetching binaryen $BINARYEN_VERSION for ${arch}-${os} (pinned by Stylus.toml)"
  curl -sL -m 300 -o "$dir/binaryen.tar.gz" "$url"
  tar xzf "$dir/binaryen.tar.gz" -C "$dir"
  export PATH="$dir/binaryen-version_${BINARYEN_VERSION}/bin:$PATH"
}

# Deploys a Stylus contract that is too large for a single code fragment onto a mined v4 hook
# address, which `cargo stylus` cannot currently do on its own.
#
# Arbitrum caps one code object at 24 KB, and lifts that for Stylus by letting a contract be split
# across up to `ArbOwnerPublic.getMaxStylusContractFragments()` of them — four, on One and Sepolia
# as of this writing, so 96 KB. What gets deployed at the contract's own address is then a *root*:
# the marker `0xEFF00200`, the uncompressed wasm size, and the addresses its fragments landed at.
#
# That is what breaks address mining. A hook's address is not free — v4 reads the callbacks it
# implements out of the low bits — so it has to be CREATE2'd from a mined salt, which needs the
# init code up front. But the init code contains the fragment addresses, and those are not known
# until the fragments are deployed. `cargo stylus get-initcode` gives up at exactly this point:
# "fragmented contracts not currently supported for initcode retrieval".
#
# The way round it is to deploy the fragments first and read the root back off the chain:
#
#   1. `cargo stylus deploy --wasm-file` — the one deploy path that does not call the constructor,
#      which matters because this contract's constructor rejects an address without its permission
#      bits, and this first deployment lands wherever it lands. It also activates, so the mined
#      copy shares its codehash and needs no activation fee of its own.
#   2. Read that contract's code. It *is* the root, verbatim.
#   3. Wrap it in the same EVM prelude `cargo stylus` would have (PUSH32 len, CODECOPY, RETURN,
#      one version byte) to get the init code, and mine a salt against it.
#   4. Send it to the StylusDeployer by hand.
#
# The first root is thrown away; the fragments it paid for are what the mined one points at.
#
# Usage: deploy_fragmented_hook <package> <permissions> <ctor-sig> <ctor-args> <deployer> <workdir>
# Echoes the mined hook address.
deploy_fragmented_hook() {
  local pkg="$1" permissions="$2" ctor_sig="$3" ctor_args="$4" deployer="$5" work="$6"
  local stylus_dir; stylus_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/stylus" && pwd)

  (cd "$stylus_dir" && cargo build --release -p "$pkg" --target wasm32-unknown-unknown) \
    >"$work/build.log" 2>&1 || { cat "$work/build.log" >&2; return 1; }
  local raw="$stylus_dir/target/wasm32-unknown-unknown/release/${pkg//-/_}.wasm"
  # The same pass `cargo stylus` applies in project mode, which `--wasm-file` mode skips.
  wasm-opt -Oz --enable-bulk-memory-opt --enable-sign-ext --enable-mutable-globals \
    --enable-nontrapping-float-to-int "$raw" -o "$work/opt.wasm" || return 1

  (cd "$stylus_dir" && cargo stylus deploy --wasm-file "$work/opt.wasm" --no-verify \
      -e "$RPC" --private-key "$KEY") >"$work/root1.log" 2>&1 \
    || { tail -20 "$work/root1.log" >&2; return 1; }
  local root1
  root1=$(grep -aoiE 'deployed code at address[^0]*0x[0-9a-fA-F]{40}' "$work/root1.log" \
    | grep -oE '0x[0-9a-fA-F]{40}' | tail -1)
  [ -n "$root1" ] || { tail -20 "$work/root1.log" >&2; return 1; }

  cast code "$root1" --rpc-url "$RPC" >"$work/root.hex"
  python3 - "$work/root.hex" "$work/initcode.hex" <<'PY'
import sys
code = bytes.fromhex(open(sys.argv[1]).read().strip().removeprefix('0x'))
assert code[:4] == bytes([0xEF, 0xF0, 0x02, 0x00]), "not a fragmented root contract"
# `DeploymentCalldata::new` in stylus-tools: PUSH32 <len> DUP1 PUSH1 43 PUSH1 0 CODECOPY
# PUSH1 0 RETURN, then one version byte, then the code. 43 is the prelude length.
prelude = b'\x7f' + len(code).to_bytes(32, 'big') + bytes([0x80, 0x60, 43, 0x60, 0x00, 0x39, 0x60, 0x00, 0xf3, 0x00])
open(sys.argv[2], 'w').write((prelude + code).hex())
PY

  (cd "$stylus_dir" && cargo run -q -p stylus-hook-miner -- \
      --initcode-file "$work/initcode.hex" --permissions "$permissions" \
      --constructor-signature "$ctor_sig" --constructor-args "$ctor_args" \
      --deployer "$deployer") >"$work/mine.log" 2>&1 || { cat "$work/mine.log" >&2; return 1; }
  local salt init_data
  salt=$(grep -m1 '^salt:' "$work/mine.log" | grep -oE '0x[0-9a-fA-F]{64}')
  init_data=$(grep -m1 '^init data:' "$work/mine.log" | grep -oE '0x[0-9a-fA-F]*')

  cast send "$deployer" 'deploy(bytes,bytes,uint256,bytes32)(address)' \
    "0x$(cat "$work/initcode.hex")" "$init_data" 0 "$salt" \
    --rpc-url "$RPC" --private-key "$KEY" >"$work/root2.log" 2>&1 \
    || { tail -20 "$work/root2.log" >&2; return 1; }

  grep -m1 '^hook address:' "$work/mine.log" | grep -oE '0x[0-9a-fA-F]{40}'
}

# Steps the dev chain up to the ArbOS version Arbitrum One is on.
#
# The published `nitro-node:*-dev` images boot on ArbOS 59, two versions behind One and Sepolia.
# That matters here because Stylus code fragments — and so any contract over 24 KB — do not exist
# before ArbOS 61: `ArbOwnerPublic.getMaxStylusContractFragments` simply reverts, and
# `cargo stylus deploy` reads that as "contract too large". The dev account owns the chain, so it
# can schedule the upgrade itself.
ensure_arbos() {
  local want="${1:-61}"
  cast send 0x0000000000000000000000000000000000000070 \
    "scheduleArbOSUpgrade(uint64,uint64)" "$want" 0 \
    --rpc-url "$RPC" --private-key "$KEY" >/dev/null 2>&1 || return 1
  # The upgrade lands on the next block, so provoke one.
  cast send 0x0000000000000000000000000000000000000070 "setL1PricePerUnit(uint256)" 0 \
    --rpc-url "$RPC" --private-key "$KEY" >/dev/null 2>&1
  cast call 0x000000000000000000000000000000000000006b \
    "getMaxStylusContractFragments()(uint16)" --rpc-url "$RPC" >/dev/null 2>&1
}

# Puts a deployed Stylus contract into Arbitrum's program cache, so its calls pay the cached init
# gas rather than the full load.
#
# On a real chain this is an auction: `CacheManager.placeBid` costs ETH, space is finite and low
# bids get evicted. A dev node has no CacheManager at all, which is why every other benchmark here
# reports the uncached figure. But the chain owner can appoint one, and the dev account owns the
# chain — so it appoints itself and caches the program directly. That turns the cached number from
# something quoted out of `ArbWasm.programInitGas` into something measured.
cache_stylus_program() {
  local program=$1 eoa=$2
  cast send 0x0000000000000000000000000000000000000070 "addWasmCacheManager(address)" "$eoa" \
    --rpc-url "$RPC" --private-key "$KEY" >/dev/null 2>&1
  cast send 0x0000000000000000000000000000000000000072 "cacheProgram(address)" "$program" \
    --rpc-url "$RPC" --private-key "$KEY" >/dev/null || return 1
  local codehash; codehash=$(cast keccak "$(cast code "$program" --rpc-url "$RPC")")
  cast call 0x0000000000000000000000000000000000000072 "codehashIsCached(bytes32)(bool)" \
    "$codehash" --rpc-url "$RPC"
}

# Builds and deploys the production TWAMM hook at a mined address.
#
# `uniswap/lib/v4-twamm-hook` is akshatmittal/v4-twamm-hook — the TWAMM written by Uniswap Labs and
# Zaha Studio, audited by ABDK Consulting and Certora, live on Base and Unichain. It is UNLICENSED,
# so it is a submodule rather than vendored, and it is built in its own checkout against its own
# pinned v4 because its source predates v4-core moving `ModifyLiquidityParams` out of
# `IPoolManager`. That only matters at the source level: the callback ABI is unchanged, so the
# binary runs against the PoolManager this benchmark deploys.
deploy_production_twamm() {
  local pool_manager=$1 interval=$2 owner=$3 create2=$4 work=$5
  local root; root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
  local dir="$root/uniswap/lib/v4-twamm-hook"
  [ -f "$dir/src/TWAMM.sol" ] || {
    echo "submodule uniswap/lib/v4-twamm-hook is not initialised" >&2; return 1; }

  (cd "$dir" && git submodule update --init --recursive --depth 1 -q \
     && forge build --skip test --skip script) >"$work/prod-build.log" 2>&1 \
    || { tail -20 "$work/prod-build.log" >&2; return 1; }

  local code args initcode
  code=$(python3 -c "import json;print(json.load(open('$dir/out/TWAMM.sol/TWAMM.json'))['bytecode']['object'])")
  args=$(cast abi-encode "f(address,uint256,address)" "$pool_manager" "$interval" "$owner")
  initcode="${code#0x}${args#0x}"
  printf '0x%s' "$initcode" >"$work/prod-initcode.hex"

  (cd "$root/stylus" && cargo run -q -p stylus-hook-miner -- \
      --initcode-file "$work/prod-initcode.hex" --plain-create2 --deployer "$create2" \
      --permissions before-initialize,before-add-liquidity,before-remove-liquidity,before-swap) \
      >"$work/prod-mine.log" 2>&1 || { cat "$work/prod-mine.log" >&2; return 1; }
  local salt; salt=$(grep -m1 '^salt:' "$work/prod-mine.log" | grep -oE '0x[0-9a-fA-F]{64}')

  cast send "$create2" "${salt}${initcode}" --rpc-url "$RPC" --private-key "$KEY" \
    >"$work/prod-deploy.log" 2>&1 || { tail -10 "$work/prod-deploy.log" >&2; return 1; }

  local mined; mined=$(grep -m1 '^hook address:' "$work/prod-mine.log" | grep -oE '0x[0-9a-fA-F]{40}')
  [ "$(cast code "$mined" --rpc-url "$RPC" | wc -c)" -gt 10 ] || {
    echo "nothing deployed at the mined address $mined" >&2; return 1; }
  echo "$mined"
}
