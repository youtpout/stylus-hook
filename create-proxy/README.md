# Deterministic Deployment Proxy
This is a proxy contract that can be deployed to any chain at the same address, and can then in turn deploy any contract at a deterministic location using CREATE2.  To use, first deploy the contract using the one-time-account transaction specified in `output/deployment.json` (or grab last known good from bottom of readme), then submit a transaction `to` the address specified in `output/deployment.json` (or grab last known good from bottom of readme). The data should be the 32 byte 'salt' followed by your init code.

## Usage
```bash
npm install
npm run build
./scripts/test.sh
```

### Details
See `scripts/test.sh` (commented).  Change `JSON_RPC` environment variable to the chain of your choice (requires an unlocked wallet with ETH).  Notice that the script works against _any_ chain!  If the chain already has the deployer contract deployed to it, then you can just comment out the deployment steps (line 20 and 23) and everything else will still function as normal.  If you have already deployed your contract with it to the chain you are pointing at, the script will fail because your contract already exists at its address.

## Explanation
This repository contains a simple contract that can deploy other contracts with a deterministic address on any chain using CREATE2.  The CREATE2 call will deploy a contract (like CREATE opcode) but instead of the address being `keccak256(rlp([deployer_address, nonce]))` it instead uses the hash of the contract's bytecode and a salt.  This means that a given deployer address will deploy the same code to the same address no matter _when_ or _where_ they issue the deployment.  The deployer is deployed with a one-time-use-account, so no matter what chain the deployer is on, its address will always be the same.  This means the only variables in determining the address of your contract are its bytecode hash and the provided salt.

Between the use of CREATE2 opcode and the one-time-use-account for the deployer, we can ensure that a given contract will exist at the _exact_ same address on every chain, but without having to use the same gas pricing or limits every time.

----

## Latest Outputs

**Note:** as of last readme update; don't trust these to be latest!

It is known to have been deployed to: [Ropsten test-net](https://ropsten.etherscan.io/tx/0xeddf9e61fb9d8f5111840daef55e5fde0041f5702856532cdbb5a02998033d26)

### Proxy Address
```
0x4e59b44847b379578588920ca78fbf26c0b4956c
```

### Deployment Transaction
```
0xf8a58085174876e800830186a08080b853604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf31ba02222222222222222222222222222222222222222222222222222222222222222a02222222222222222222222222222222222222222222222222222222222222222
```

### Deployment Signer Address
```
0x3fab184622dc19b6109349b94811493bf2a45362
```

### Deployment Gas Price
```
100 nanoeth (gwei)
```

### Deployment Gas Limit
```
100000
```

**Note:** The actual gas used is 68131, but that can change if the cost of opcodes changes.  To avoid having to move the proxy to a different address, we opted to give excess gas.  Given the gas price, this may result in notable expenses, but since this only needs to be deployed once per chain that is acceptable.
