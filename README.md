# stylus-hook

## Launch node

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

## Deploy stylus contract on local
// 0x14791697260E4c9A71f18484C9f997B308e59325
Private Key for test 0x0123456789012345678901234567890123456789012345678901234567890123 

```bash
cd /hook
cargo stylus deploy --private-key 0x0123456789012345678901234567890123456789012345678901234567890123 -e http://localhost:8547/
```


## WSL Issue with nitro

[Install docker](https://dev.to/kenji_goh/got-permission-denied-while-trying-to-connect-to-the-docker-daemon-socket-3dne)