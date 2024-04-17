import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";
import { IPoolManager, PoolSwapTest, PoolSwapTest__factory, TickMath } from "../typechain";
import { PoolKeyStruct } from "../typechain/BaseHook";
// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";
const addressContract = "0xcC946789cD835EeeD198c3A39A5B1A7C76b5C044";
const abi = [{ "type": "constructor", "stateMutability": "nonpayable", "inputs": [{ "type": "address", "name": "_manager", "internalType": "contract IPoolManager" }] }, { "type": "error", "name": "ERC20TransferFailed", "inputs": [] }, { "type": "error", "name": "NativeTransferFailed", "inputs": [] }, { "type": "function", "stateMutability": "view", "outputs": [{ "type": "address", "name": "", "internalType": "contract IPoolManager" }], "name": "manager", "inputs": [] }, { "type": "function", "stateMutability": "payable", "outputs": [{ "type": "int256", "name": "delta", "internalType": "BalanceDelta" }], "name": "modifyLiquidity", "inputs": [{ "type": "tuple", "name": "key", "internalType": "struct PoolKey", "components": [{ "type": "address", "name": "currency0", "internalType": "Currency" }, { "type": "address", "name": "currency1", "internalType": "Currency" }, { "type": "uint24", "name": "fee", "internalType": "uint24" }, { "type": "int24", "name": "tickSpacing", "internalType": "int24" }, { "type": "address", "name": "hooks", "internalType": "contract IHooks" }] }, { "type": "tuple", "name": "params", "internalType": "struct IPoolManager.ModifyLiquidityParams", "components": [{ "type": "int24", "name": "tickLower", "internalType": "int24" }, { "type": "int24", "name": "tickUpper", "internalType": "int24" }, { "type": "int256", "name": "liquidityDelta", "internalType": "int256" }] }, { "type": "bytes", "name": "hookData", "internalType": "bytes" }, { "type": "bool", "name": "settleUsingTransfer", "internalType": "bool" }, { "type": "bool", "name": "withdrawTokens", "internalType": "bool" }] }, { "type": "function", "stateMutability": "payable", "outputs": [{ "type": "int256", "name": "delta", "internalType": "BalanceDelta" }], "name": "modifyLiquidity", "inputs": [{ "type": "tuple", "name": "key", "internalType": "struct PoolKey", "components": [{ "type": "address", "name": "currency0", "internalType": "Currency" }, { "type": "address", "name": "currency1", "internalType": "Currency" }, { "type": "uint24", "name": "fee", "internalType": "uint24" }, { "type": "int24", "name": "tickSpacing", "internalType": "int24" }, { "type": "address", "name": "hooks", "internalType": "contract IHooks" }] }, { "type": "tuple", "name": "params", "internalType": "struct IPoolManager.ModifyLiquidityParams", "components": [{ "type": "int24", "name": "tickLower", "internalType": "int24" }, { "type": "int24", "name": "tickUpper", "internalType": "int24" }, { "type": "int256", "name": "liquidityDelta", "internalType": "int256" }] }, { "type": "bytes", "name": "hookData", "internalType": "bytes" }] }, { "type": "function", "stateMutability": "nonpayable", "outputs": [{ "type": "bytes", "name": "", "internalType": "bytes" }], "name": "unlockCallback", "inputs": [{ "type": "bytes", "name": "rawData", "internalType": "bytes" }] }];


const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const swapRouter = PoolSwapTest__factory.connect("0x4aa4365da82ACD46e378A6f3c92a863f3e763d34", signer);



// 100 $
const amount = BigInt(1000) ** BigInt(18);
let zeroForOne = true;

console.log("amount", amount);
// two swaps
await swap(amount, zeroForOne);

async function swap(amount, zeroForOne) {
    const MUNI_ADDRESS = "0x2dC942dcba13E4BE27721980FE01f2221610A93b";
    const MUSDC_ADDRESS = "0xFB9a956c4875826a76d47C360234FC1633C078A8";
    const hooks = "0x2b02aBF3572e805547A7029f304aA6bbf6e0031F";
    const MIN_PRICE_LIMIT = BigInt(4295128740);
    const MAX_PRICE_LIMIT = BigInt(1461446703485210103287273052203988822378723970341);

    const poolKey: PoolKeyStruct = { currency0: MUNI_ADDRESS, currency1: MUSDC_ADDRESS, fee: 3000, tickSpacing: 60, hooks };
    const swapParams: IPoolManager.SwapParamsStruct = { zeroForOne, amountSpecified: amount, sqrtPriceLimitX96: zeroForOne ? MIN_PRICE_LIMIT : MAX_PRICE_LIMIT };
    const testSettings: PoolSwapTest.TestSettingsStruct = { withdrawTokens: true, settleUsingTransfer: true, currencyAlreadySent: false };

    const tx = await swapRouter.swap(poolKey, swapParams, testSettings, ethers.ZeroHash);
    const res = await tx.wait();
    console.log("result", res);

}