import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";
import { IPoolManager, PoolSwapTest, PoolSwapTest__factory, TickMath, Token, Token__factory } from "../typechain";
import { PoolKeyStruct } from "../typechain/BaseHook";
// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";

const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const swapRouter: PoolSwapTest = PoolSwapTest__factory.connect("0xFB9a956c4875826a76d47C360234FC1633C078A8", signer);

const MUNI_ADDRESS = "0xf3e1C2DefcDfE770972D4bCF45B03498626c5594";
const MUSDC_ADDRESS = "0xADeb1300E4860089d93233ddED31B33206ba8432";

// 100 $
const amount = BigInt(100) * (BigInt(10) ** BigInt(18));
let zeroForOne = true;

console.log("amount", amount);
const hooks = "0x010dB0326D1A8ddEC5C7daAffd8f84F9A367394D";
const hooksProxy = "0x0105aC59AdaBE5CD26ce684e309C1aa5D9D93874";
const token0: Token = Token__factory.connect(MUNI_ADDRESS, signer);
const token1: Token = Token__factory.connect(MUSDC_ADDRESS, signer);
const approveAmount = BigInt(10000000) * (BigInt(10) ** BigInt(18));
let tx1 = await token0.approve("0xFB9a956c4875826a76d47C360234FC1633C078A8", approveAmount);
await tx1.wait();
tx1 = await token1.approve("0xFB9a956c4875826a76d47C360234FC1633C078A8", approveAmount);
await tx1.wait();
// two swaps
await swap(amount, zeroForOne, hooks);
await swap(amount, zeroForOne, hooksProxy);

await swap(amount, zeroForOne, hooks);
await swap(amount, zeroForOne, hooksProxy);

async function swap(amount, zeroForOne, hooks) {
    const MIN_PRICE_LIMIT = BigInt(4295128740);
    const MAX_PRICE_LIMIT = BigInt(1461446703485210103287273052203988822378723970341);

    const poolKey: PoolKeyStruct = { currency0: MUSDC_ADDRESS, currency1: MUNI_ADDRESS, fee: 3000, tickSpacing: 60, hooks };
    const swapParams: IPoolManager.SwapParamsStruct = { zeroForOne, amountSpecified: amount, sqrtPriceLimitX96: zeroForOne ? MIN_PRICE_LIMIT : MAX_PRICE_LIMIT };
    const testSettings: PoolSwapTest.TestSettingsStruct = { withdrawTokens: true, settleUsingTransfer: true, currencyAlreadySent: false };

    const tx = await swapRouter.swap(poolKey, swapParams, testSettings, ethers.ZeroHash, { gasLimit: 1000000 });
    const res = await tx.wait();
    console.log("result", res);
}