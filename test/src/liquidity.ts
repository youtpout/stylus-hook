import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";
import { IPoolManager, PoolModifyLiquidityTest__factory, PoolModifyLiquidityTest, Token, Token__factory } from "../typechain";
import { PoolKeyStruct } from "../typechain/PoolModifyLiquidityTest";

// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";

const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const MUNI_ADDRESS = "0xf3e1C2DefcDfE770972D4bCF45B03498626c5594";
const MUSDC_ADDRESS = "0xADeb1300E4860089d93233ddED31B33206ba8432";

const hooks = "0x010dB0326D1A8ddEC5C7daAffd8f84F9A367394D";
const hooksProxy = "0x010F26be5b68Cd83c0534cEe84C173818244B12f";

const lpRouter: PoolModifyLiquidityTest = PoolModifyLiquidityTest__factory.connect("0x2dC942dcba13E4BE27721980FE01f2221610A93b", signer);
const token0: Token = Token__factory.connect(MUNI_ADDRESS, signer);
const token1: Token = Token__factory.connect(MUSDC_ADDRESS, signer);
const approveAmount = BigInt(10000000) * (BigInt(10) ** BigInt(18));
let tx1 = await token0.approve("0x2dC942dcba13E4BE27721980FE01f2221610A93b", approveAmount);
await tx1.wait();
tx1 = await token1.approve("0x2dC942dcba13E4BE27721980FE01f2221610A93b", approveAmount);
await tx1.wait();

await modifyLiquidity(hooks);
await modifyLiquidity(hooksProxy);

async function modifyLiquidity(hooks) {
    const MIN_PRICE_LIMIT = BigInt(4295128740);
    const MAX_PRICE_LIMIT = BigInt(1461446703485210103287273052203988822378723970341);

    const poolKey: PoolKeyStruct = { currency0: MUSDC_ADDRESS, currency1: MUNI_ADDRESS, fee: 3000, tickSpacing: 60, hooks };
    const liquidityParams: IPoolManager.ModifyLiquidityParamsStruct = { tickLower: -600, tickUpper: 600, liquidityDelta: (approveAmount / BigInt(10)) };
    const tx = await lpRouter.modifyLiquidity(poolKey, liquidityParams, ethers.ZeroHash);
    const res = await tx.wait();
    console.log("result", res);
}

