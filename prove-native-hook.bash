#!/usr/bin/env bash
# Proves that a Uniswap v4 hook written entirely in Rust works — not that its ABI looks right, but
# that a real PoolManager calls it and it counts what it is supposed to count.
#
#   ./prove-native-hook.bash
#
# Stands up an Arbitrum Nitro dev node, deploys v4 onto it, mines a CREATE2 salt so the Stylus
# contract lands on an address carrying its permission flags, deploys it there, then opens a pool on
# it and swaps. No Solidity hook anywhere in the path.
set -euo pipefail

NITRO_IMAGE=${NITRO_IMAGE:-offchainlabs/nitro-node:v3.11.3-beb2108-dev}
CONTAINER=${CONTAINER:-stylus-native-proof}
RPC=${RPC:-http://127.0.0.1:8547}
KEY=0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
ARB_OWNER=0x0000000000000000000000000000000000000070
ARB_WASM=0x0000000000000000000000000000000000000071
LIQUIDITY=1000000000000000000000
SWAP_AMOUNT=1000000000000000000

ROOT=$(cd "$(dirname "$0")" && pwd)
WORK=$(mktemp -d)
export PATH="$HOME/.foundry/bin:$PATH"
# shellcheck source=bench-lib.bash
. "$ROOT/bench-lib.bash"
ensure_binaryen "$WORK"

log()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
ok()   { printf '\033[32m  ok\033[0m  %s\n' "$*"; }
fail() { printf '\033[31m  FAIL\033[0m %s\n' "$*"; exit 1; }

log "starting a fresh nitro dev node"
docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
docker run --rm -d --name "$CONTAINER" -p 8547:8547 "$NITRO_IMAGE" \
  --dev --http.addr 0.0.0.0 --http.port 8547 --http.api=net,web3,eth,debug,arb \
  --http.corsdomain='*' --http.vhosts='*' >/dev/null
trap 'docker rm -f "$CONTAINER" >/dev/null 2>&1 || true; rm -rf "$WORK"' EXIT
for _ in $(seq 1 60); do cast chain-id --rpc-url "$RPC" >/dev/null 2>&1 && break; sleep 1; done
echo "chain $(cast chain-id --rpc-url "$RPC"), stylus v$(cast call $ARB_WASM 'stylusVersion()(uint16)' --rpc-url "$RPC")"

# forge estimates gas without Arbitrum's L1 data component and then fails to broadcast
cast send $ARB_OWNER "setL1PricePerUnit(uint256)" 0 --rpc-url "$RPC" --private-key $KEY >/dev/null

# Permit2 is deployed with CREATE2, and the canonical factory cannot be installed from its presigned
# transaction on Arbitrum — its 100k gas limit is below Arbitrum's intrinsic gas. Deploy a copy.
CREATE2_DEPLOYER=$(cast send --rpc-url "$RPC" --private-key $KEY --create \
  0x604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf3 \
  | awk '/^contractAddress/{print $2}')

log "deploying v4, the tokens and a StylusDeployer"
(cd "$ROOT/uniswap" && forge script script/bench/DeployNativeFixture.s.sol:DeployNativeFixtureScript \
  --rpc-url "$RPC" --private-key $KEY --create2-deployer "$CREATE2_DEPLOYER" \
  --gas-estimate-multiplier 200 --broadcast --slow) \
  >"$WORK/deploy.log" 2>&1 || { tail -40 "$WORK/deploy.log"; exit 1; }

addr_of() { grep -m1 "^  $1" "$WORK/deploy.log" | grep -o '0x[0-9a-fA-F]\{40\}' || true; }
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
grep -E "^  [a-zA-Z].*0x" "$WORK/deploy.log" | sed 's/^  //'

log "mining a hook address for the Rust contract"
(cd "$ROOT/stylus" && cargo stylus get-initcode --contract stylus-native-counter) 2>/dev/null \
  | tail -1 >"$WORK/initcode.hex"
CONSTRUCTOR=$( (cd "$ROOT/stylus" && cargo stylus constructor --contract stylus-native-counter) 2>/dev/null | tail -1)
echo "constructor: $CONSTRUCTOR"

(cd "$ROOT/stylus" && cargo run -q -p stylus-hook-miner -- \
  --initcode-file "$WORK/initcode.hex" \
  --permissions before-swap,after-swap,before-add-liquidity,before-remove-liquidity \
  --constructor-signature "$CONSTRUCTOR" \
  --constructor-args "$POOL_MANAGER" \
  --deployer "$STYLUS_DEPLOYER") >"$WORK/mine.log" 2>&1 || { cat "$WORK/mine.log"; exit 1; }
MINED=$(grep -m1 'hook address' "$WORK/mine.log" | grep -o '0x[0-9a-fA-F]\{40\}' || true)
SALT=$(grep -m1 '^salt:' "$WORK/mine.log" | grep -o '0x[0-9a-fA-F]\{64\}' || true)
sed -n '1,4p' "$WORK/mine.log"

log "deploying the Rust hook to the mined address"
(cd "$ROOT/stylus" && cargo stylus deploy --contract stylus-native-counter --no-verify \
  -e "$RPC" --private-key $KEY \
  --deployer-address "$STYLUS_DEPLOYER" --deployer-salt "$SALT" \
  --constructor-args "$POOL_MANAGER") >"$WORK/stylus-deploy.log" 2>&1 \
  || { tail -30 "$WORK/stylus-deploy.log"; exit 1; }
HOOK=$(grep -aoiE '(contract deployed at address|deployed code at address)[^0]*0x[0-9a-fA-F]{40}' \
  "$WORK/stylus-deploy.log" | grep -oE '0x[0-9a-fA-F]{40}' | tail -1 || true)
if [ -z "$HOOK" ]; then
  echo "could not find the deployed address in:"; tail -30 "$WORK/stylus-deploy.log"; exit 1
fi
echo "deployed at: $HOOK"

[ "$(echo "$HOOK" | tr 'A-Z' 'a-z')" = "$(echo "$MINED" | tr 'A-Z' 'a-z')" ] \
  && ok "landed on the mined address" \
  || fail "expected $MINED, got $HOOK"

[ "$(cast code "$HOOK" --rpc-url "$RPC" | head -c 10)" = "0xeff00000" ] \
  && ok "the code at that address is an activated Stylus program" \
  || fail "no Stylus program at $HOOK"

[ "$(cast call "$HOOK" 'poolManager()(address)' --rpc-url "$RPC" | tr 'A-Z' 'a-z')" \
    = "$(echo "$POOL_MANAGER" | tr 'A-Z' 'a-z')" ] \
  && ok "its constructor ran and validated the address" \
  || fail "constructor did not run"

log "opening a pool on it and adding liquidity"
# this is beforeAddLiquidity: the PoolManager calling a WASM contract
cast send "$FIXTURE" "open(address,address,address,uint128)" \
  "$CURRENCY0" "$CURRENCY1" "$HOOK" "$LIQUIDITY" \
  --rpc-url "$RPC" --private-key $KEY >/dev/null
POOL_ID=$(cast call "$FIXTURE" 'poolId(uint256)(bytes32)' 0 --rpc-url "$RPC")
echo "pool: $POOL_ID"

count() { cast call "$HOOK" "$1(bytes32)(uint256)" "$POOL_ID" --rpc-url "$RPC" | awk '{print $1}'; }

[ "$(count beforeAddLiquidity"Count")" = "1" ] \
  && ok "beforeAddLiquidity reached the Rust hook" \
  || fail "beforeAddLiquidity count is $(count beforeAddLiquidityCount), expected 1"

log "swapping through it"
cast send "$FIXTURE" "swap(uint256,uint256,bool)" 0 "$SWAP_AMOUNT" true \
  --rpc-url "$RPC" --private-key $KEY >/dev/null
[ "$(count beforeSwapCount)" = "1" ] && ok "beforeSwap reached the Rust hook" \
  || fail "beforeSwap count is $(count beforeSwapCount), expected 1"
[ "$(count afterSwapCount)" = "1" ] && ok "afterSwap reached the Rust hook" \
  || fail "afterSwap count is $(count afterSwapCount), expected 1"

cast send "$FIXTURE" "swap(uint256,uint256,bool)" 0 "$SWAP_AMOUNT" false \
  --rpc-url "$RPC" --private-key $KEY >/dev/null
[ "$(count afterSwapCount)" = "2" ] && ok "a second swap counted again" \
  || fail "afterSwap count is $(count afterSwapCount), expected 2"

log "removing liquidity"
cast send "$FIXTURE" "removeLiquidity(uint256,uint128)" 0 1000000000000000000 \
  --rpc-url "$RPC" --private-key $KEY >/dev/null
[ "$(count beforeRemoveLiquidityCount)" = "1" ] \
  && ok "beforeRemoveLiquidity reached the Rust hook" \
  || fail "beforeRemoveLiquidity count is $(count beforeRemoveLiquidityCount), expected 1"

log "a callback the hook did not declare stays unimplemented"
cast call "$HOOK" "beforeDonate(address,(address,address,uint24,int24,address),uint256,uint256,bytes)" \
  "$POOL_MANAGER" "($CURRENCY0,$CURRENCY1,3000,60,$HOOK)" 0 0 0x --rpc-url "$RPC" >/dev/null 2>&1 \
  && fail "beforeDonate should have reverted" \
  || ok "beforeDonate reverts with HookNotImplemented"

log "a Uniswap v4 hook with no Solidity in it works"
printf 'hook   %s\n' "$HOOK"
printf 'pool   %s\n' "$POOL_ID"
printf 'swaps  %s before, %s after\n' "$(count beforeSwapCount)" "$(count afterSwapCount)"
