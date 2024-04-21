import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";
import { AirdropHook, AirdropHookProxy, AirdropHookProxy__factory, AirdropHook__factory, IPoolManager, PoolSwapTest, PoolSwapTest__factory, TickMath, Token, Token__factory } from "../typechain";
import { PoolKeyStruct } from "../typechain/BaseHook";
// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";

const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const hooks = "0x010B3f77C446FAafDfdEb26fE32A2436fA703919";
const hooksProxy = "0x010F26be5b68Cd83c0534cEe84C173818244B12f";
const poolId = "0xdae3384441866b1527f547e718ee3436354b666ed2f7575340d4cd8d0945b98f";
const poolIdProxy = "0x02a8167316a4e129c897cf0ccad97ce9c9ffc970f40d451202d772b7aff9b313";

const aidropHook: AirdropHook = AirdropHook__factory.connect(hooks, signer);
const aidropHookProxy: AirdropHookProxy = AirdropHookProxy__factory.connect(hooksProxy, signer);

const tx11 = await aidropHook.closeAirdrop(poolId, "0x878873B6E9ebe8Fd22816A69B51e6D96cE75961E");
const res11 = await tx11.wait();

const tx22 = await aidropHookProxy.closeAirdrop(poolIdProxy, "0xb2f902825D87efEE4E3eF6873b071F7FA86ca9aB");
const res22 = await tx22.wait();


const tx = await aidropHook.claimAirdrop(poolId);
const res = await tx.wait();

const tx2 = await aidropHookProxy.claimAirdrop(poolIdProxy);
const res2 = await tx2.wait();