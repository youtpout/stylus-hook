# stylus-hook

## Launch node

Define a secret key for blockscout folder /hook/nitro-tesnode/blockscout/nitro.env
```
SECRET_KEY_BASE=VTIB3uHDNbvrY0+60ZWgUoUBKDn9ppLR8MI4CpRz4/qLyEFs54ktJfaNT6Z221No
```

First launch
```bash
cd /hook/nitro-testnode
./test-node.bash --init --blockscout
```

Don't use init if you want to preserve data to next launch
```bash
./test-node.bash --blockscout
```

Add some ethereum to the test account
```bash
./test-node.bash script send-l2 --to address_0x14791697260E4c9A71f18484C9f997B308e59325 --ethamount 5
```

## Deploy uniswap v4
Deploy on local node
```bash
cd /v4-template
forge script script/DeployHook.s.sol:DeployHookScript --rpc-url localhost --broadcast -vvvvv 
```

Verify contract example
```bash
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x438aa22765874199d7ae86CC3f4622a5e41Fe1c4 PoolManager --constructor-args $(cast abi-encode "constructor(uint256)" 500000) 

forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x2c4bFd0fBC64096D24e8Cb78dcB57eF5518Bb626 PoolManager --constructor-args $(cast abi-encode "constructor(uint256)" 500000) 
```



## Deploy create2 proxy for uniswap
```bash
./create-proxy/scripts/deploy.sh
```

## Deploy stylus contract on local
// 0x14791697260E4c9A71f18484C9f997B308e59325
Private Key for test 0x0123456789012345678901234567890123456789012345678901234567890123 

```bash
cd /hook
cargo stylus deploy --private-key 0x0123456789012345678901234567890123456789012345678901234567890123 -e http://localhost:8547/
```


## WSL Issue with nitro

[Install docker](https://dev.to/kenji_goh/got-permission-denied-while-trying-to-connect-to-the-docker-daemon-socket-3dne)