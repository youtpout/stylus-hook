import { Signature, ethers, Wallet, BaseWallet, SigningKey, Contract } from "ethers";
import { IPoolManager, PoolSwapTest, PoolSwapTest__factory, TickMath, Redeploy__factory } from "../typechain";
import { PoolKeyStruct } from "../typechain/BaseHook";
// replace by your configuration
const url = "http://localhost:8547/";
// address 0x14791697260E4c9A71f18484C9f997B308e59325
const privateKey = "0x0123456789012345678901234567890123456789012345678901234567890123";
const addressContract = "0x76Eec016f5bB4360BCA1425E26A8Af360D3793f5";
const hookStylus = "0xb23CB9b2EDfB2DC0F4Fd0C5c65406bB4F9e2883d";

// calculate correct salt for hook address
const provider = new ethers.JsonRpcProvider(url);
const signer = new Wallet(privateKey, provider);

const redeployer = Redeploy__factory.connect(addressContract, signer);
const expectStart = "0x2b0";

for (let index = 0; index < 1000000; index++) {
    const salt = ethers.zeroPadValue(ethers.toBeHex(BigInt(index)), 32);
    console.log("found salt : ", salt);
    const calculate = await redeployer.predictDeterministicAddress(hookStylus, salt);

    console.log("calculated : ", calculate);
    // check if address start with desired flags
    if (calculate.toLowerCase().indexOf(expectStart) > -1) {
        console.log("found salt : ", salt);
        const tx = await redeployer.deployDeterministic(hookStylus, salt);
        const res = await tx.wait();

        console.log("deployed at : ", calculate);
        break;
    }

}
