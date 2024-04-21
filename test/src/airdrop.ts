import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";
import { AirdropHook, AirdropHookProxy, AirdropHookProxy__factory, AirdropHook__factory, IAirdropHook, IAirdropHook__factory, IPoolManager, PoolSwapTest, PoolSwapTest__factory, TickMath, Token, Token__factory } from "../typechain";
import { PoolKeyStruct } from "../typechain/BaseHook";
// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";

const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const hooks = "0x010dB0326D1A8ddEC5C7daAffd8f84F9A367394D";
const stylus = "0x8cDE56336E289c028C8f7CF5c20283fF02272182";
const hooksProxy = "0x0105aC59AdaBE5CD26ce684e309C1aa5D9D93874";
const poolId = "0x589bab4662b56c6c9633b45c2fd9050e0051e7e77f46de83e295a6910ca00512";
const poolIdProxy = "0xeb801fb1378a7f687ca4f548d6c76844a84f2d015ae2a08f3867fd1cd112dc6f";

const aidropHook: AirdropHook = AirdropHook__factory.connect(hooks, signer);
const aidropHookProxy: AirdropHookProxy = AirdropHookProxy__factory.connect(hooksProxy, signer);
const stylusContract: IAirdropHook = IAirdropHook__factory.connect(stylus, signer);

const tx11 = await aidropHook.closeAirdrop(poolId, "0x878873B6E9ebe8Fd22816A69B51e6D96cE75961E");
const res11 = await tx11.wait();

const tx22 = await aidropHookProxy.closeAirdrop(poolIdProxy, "0xb2f902825D87efEE4E3eF6873b071F7FA86ca9aB");
const res22 = await tx22.wait();


const tx = await aidropHook.claimAirdrop(poolId);
const res = await tx.wait();

// direct claim from stylus
const tx2 = await stylusContract.claim(poolIdProxy);
const res2 = await tx2.wait();