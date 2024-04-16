import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";

// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";
const addressContract = "0xcC946789cD835EeeD198c3A39A5B1A7C76b5C044";
const abi = [{ "type": "constructor", "stateMutability": "nonpayable", "inputs": [{ "type": "address", "name": "_manager", "internalType": "contract IPoolManager" }] }, { "type": "error", "name": "ERC20TransferFailed", "inputs": [] }, { "type": "error", "name": "NativeTransferFailed", "inputs": [] }, { "type": "function", "stateMutability": "view", "outputs": [{ "type": "address", "name": "", "internalType": "contract IPoolManager" }], "name": "manager", "inputs": [] }, { "type": "function", "stateMutability": "payable", "outputs": [{ "type": "int256", "name": "delta", "internalType": "BalanceDelta" }], "name": "modifyLiquidity", "inputs": [{ "type": "tuple", "name": "key", "internalType": "struct PoolKey", "components": [{ "type": "address", "name": "currency0", "internalType": "Currency" }, { "type": "address", "name": "currency1", "internalType": "Currency" }, { "type": "uint24", "name": "fee", "internalType": "uint24" }, { "type": "int24", "name": "tickSpacing", "internalType": "int24" }, { "type": "address", "name": "hooks", "internalType": "contract IHooks" }] }, { "type": "tuple", "name": "params", "internalType": "struct IPoolManager.ModifyLiquidityParams", "components": [{ "type": "int24", "name": "tickLower", "internalType": "int24" }, { "type": "int24", "name": "tickUpper", "internalType": "int24" }, { "type": "int256", "name": "liquidityDelta", "internalType": "int256" }] }, { "type": "bytes", "name": "hookData", "internalType": "bytes" }, { "type": "bool", "name": "settleUsingTransfer", "internalType": "bool" }, { "type": "bool", "name": "withdrawTokens", "internalType": "bool" }] }, { "type": "function", "stateMutability": "payable", "outputs": [{ "type": "int256", "name": "delta", "internalType": "BalanceDelta" }], "name": "modifyLiquidity", "inputs": [{ "type": "tuple", "name": "key", "internalType": "struct PoolKey", "components": [{ "type": "address", "name": "currency0", "internalType": "Currency" }, { "type": "address", "name": "currency1", "internalType": "Currency" }, { "type": "uint24", "name": "fee", "internalType": "uint24" }, { "type": "int24", "name": "tickSpacing", "internalType": "int24" }, { "type": "address", "name": "hooks", "internalType": "contract IHooks" }] }, { "type": "tuple", "name": "params", "internalType": "struct IPoolManager.ModifyLiquidityParams", "components": [{ "type": "int24", "name": "tickLower", "internalType": "int24" }, { "type": "int24", "name": "tickUpper", "internalType": "int24" }, { "type": "int256", "name": "liquidityDelta", "internalType": "int256" }] }, { "type": "bytes", "name": "hookData", "internalType": "bytes" }] }, { "type": "function", "stateMutability": "nonpayable", "outputs": [{ "type": "bytes", "name": "", "internalType": "bytes" }], "name": "unlockCallback", "inputs": [{ "type": "bytes", "name": "rawData", "internalType": "bytes" }] }];


const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const lpRouter = new Contract(addressContract, abi, signer);

await lpRouter.modifyLiquidity(
    [0x715b1228f5ca70329b25254cb140bfe28c6265ae, 0x8f432d45cc8c546ff104fb1df0e2fe03a3963db8, 3000, 60, 0x2b0817e76b731333be0b06989aab432866334def)],
[-600, 600, 10000000000000000000000],
    0x00000000000000000000000000000000000000000000000000000000661e48e7
);