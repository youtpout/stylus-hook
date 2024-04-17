# deploys all necessary contracts
./nitro-testnode/test-node.bash script send-l2 --to address_0x14791697260E4c9A71f18484C9f997B308e59325 --ethamount 5
./nitro-testnode/test-node.bash script send-l2 --to address_0x3fab184622dc19b6109349b94811493bf2a45362 --ethamount 5
./create-proxy/scripts/deploy.sh
cd uniswap
forge script script/DeployHook.s.sol:DeployHookScript --rpc-url localhost --broadcast -vvvvv 
# verify deployed contracts
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x8cde56336e289c028c8f7cf5c20283ff02272182 PoolManager --constructor-args $(cast abi-encode "constructor(uint256)" 500000) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x392ae47ebe5d65c0ba7560e7c46c3d167a588672 PoolModifyLiquidityTest --constructor-args $(cast abi-encode "constructor(address)" 0x8cde56336e289c028c8f7cf5c20283ff02272182) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x4aa4365da82acd46e378a6f3c92a863f3e763d34 PoolSwapTest --constructor-args $(cast abi-encode "constructor(address)" 0x8cde56336e289c028c8f7cf5c20283ff02272182) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x2dC942dcba13E4BE27721980FE01f2221610A93b Token --constructor-args $(cast abi-encode "constructor(string,string,address)" "MUNI" "MUNI" 0x14791697260E4c9A71f18484C9f997B308e59325) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0xFB9a956c4875826a76d47C360234FC1633C078A8 Token --constructor-args $(cast abi-encode "constructor(string,string,address)" "MUSDC" "MUSDC" 0x14791697260E4c9A71f18484C9f997B308e59325) --force

