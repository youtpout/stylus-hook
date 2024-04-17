# stylus-hook

Clone with all submodules

```bash
git clone --recurse-submodules https://github.com/youtpout/stylus-hook
```

If you forget submodules
```bash
git submodule update --init --recursive
```

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

## Deploy uniswap on create2 proxy
```bash
./deploy.bash
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