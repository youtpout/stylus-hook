# deploys all necessary contracts
./nitro-testnode/test-node.bash script send-l2 --to address_0x14791697260E4c9A71f18484C9f997B308e59325 --ethamount 5
./nitro-testnode/test-node.bash script send-l2 --to address_0x3fab184622dc19b6109349b94811493bf2a45362 --ethamount 5
./create-proxy/scripts/deploy.sh
cd airdrop-stylus-proxy
cargo stylus deploy --private-key 0x0123456789012345678901234567890123456789012345678901234567890123 -e http://localhost:8547/
cd ../uniswap
forge script script/DeployAirdropHook.s.sol:DeployHookScript --rpc-url localhost --broadcast -vvvvv 
# verify deployed contracts
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x4aa4365da82ACD46e378A6f3c92a863f3e763d34 PoolManager --constructor-args $(cast abi-encode "constructor(uint256)" 500000) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x2dC942dcba13E4BE27721980FE01f2221610A93b PoolModifyLiquidityTest --constructor-args $(cast abi-encode "constructor(address)" 0x4aa4365da82ACD46e378A6f3c92a863f3e763d34) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0xFB9a956c4875826a76d47C360234FC1633C078A8 PoolSwapTest --constructor-args $(cast abi-encode "constructor(address)" 0x4aa4365da82ACD46e378A6f3c92a863f3e763d34) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0xf3e1C2DefcDfE770972D4bCF45B03498626c5594 Token --constructor-args $(cast abi-encode "constructor(string,string,address)" "MUNI" "MUNI" 0x14791697260E4c9A71f18484C9f997B308e59325) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0xADeb1300E4860089d93233ddED31B33206ba8432 Token --constructor-args $(cast abi-encode "constructor(string,string,address)" "MUSDC" "MUSDC" 0x14791697260E4c9A71f18484C9f997B308e59325) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x010B3f77C446FAafDfdEb26fE32A2436fA703919 AirdropHook --constructor-args $(cast abi-encode "constructor(address)" 0x4aa4365da82ACD46e378A6f3c92a863f3e763d34) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x010F26be5b68Cd83c0534cEe84C173818244B12f AirdropHookProxy --constructor-args $(cast abi-encode "constructor(address)" 0x4aa4365da82ACD46e378A6f3c92a863f3e763d34) --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x878873B6E9ebe8Fd22816A69B51e6D96cE75961E AirdropToken --constructor-args $(cast abi-encode "constructor(string,string,address,address,uint256)" "Flydrop" "FLY" 0x14791697260E4c9A71f18484C9f997B308e59325 0x010B3f77C446FAafDfdEb26fE32A2436fA703919 50000000000000000000000000)  --force
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0xb2f902825D87efEE4E3eF6873b071F7FA86ca9aB AirdropToken --constructor-args $(cast abi-encode "constructor(string,string,address,address,uint256)" "FlydropProxy" "FLYP" 0x14791697260E4c9A71f18484C9f997B308e59325 0x010F26be5b68Cd83c0534cEe84C173818244B12f 50000000000000000000000000)  --force
# define hook address in stylus contract
cd ../airdrop-stylus-proxy
cargo run --example counter
# set liquidity and make first swap
cd ../test
npm run liquidity
npm run swap