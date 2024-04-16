import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";

// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";
const lpRouter = "0x8cDE56336E289c028C8f7CF5c20283fF02272182";
const abi = ["function number() external view returns (uint256)",
    "function setNumber(uint256 number) external",
    "function increment() external"];


const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const contract = new Contract(counterContract, abi, signer);
let number = await contract.getFunction("number").call(null);
console.log("number", number);

await contract.increment();

number = await contract.getFunction("number").call(null);
console.log("number after increment", number);

lpRouter.modifyLiquidity(
    key,
    IPoolManager.ModifyLiquidityParams(-600, 600, 10_000e18),
    hookData
);