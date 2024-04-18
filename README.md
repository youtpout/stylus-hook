# stylus-hook

Clone with all submodules

```bash
git clone --recurse-submodules https://github.com/youtpout/stylus-hook
```

If you forget submodules
```bash
git pull --recurse-submodules
```

or

```bash
git submodule update --init --recursive
```

## dependencies
You need docker, stylus and foundry
[Docker](https://docs.docker.com/engine/install/) 
[Stylus](https://docs.arbitrum.io/stylus/stylus-quickstart)
[Foundry](https://book.getfoundry.sh/getting-started/installation)

## Launch node

First launch, remove all previous data
```bash
./first-launch.bash
```
Classic launch will restore all previous transactions
```bash
./launch.bash
```

Add some ethereum to the an address
```bash
./nitro-testnode/test-node.bash script send-l2 --to address_0xyouraddress --ethamount 5
```

if you have file error access use the good old one chmod
```bash
chmod 777 nitro-testnode/test-node.bash
chmod 777 first-launch.bash 
```

## Deploy uniswap and create2 proxy
```bash
./deploy.bash
```

or 
```bash
./create-proxy/scripts/deploy.sh
```

```bash
cd /uniswap
forge script script/DeployHook.s.sol:DeployHookScript --rpc-url localhost --broadcast -vvvvv 
forge script script/Redeploy.s.sol:RedeployScript --rpc-url localhost --broadcast -vvvvv 
forge verify-contract  --verifier blockscout --verifier-url http://127.0.0.1:4000/api?  0x76Eec016f5bB4360BCA1425E26A8Af360D3793f5 Redeploy --force
```


Make a swap
```bash
cd /uniswap
forge script script/03_Swap.s.sol:SwapScript --rpc-url localhost --broadcast -vvvvv 
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