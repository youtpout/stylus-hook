/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import {
  Contract,
  ContractFactory,
  ContractTransactionResponse,
  Interface,
} from "ethers";
import type { Signer, ContractDeployTransaction, ContractRunner } from "ethers";
import type { NonPayableOverrides } from "../../common";
import type {
  OutOfBoundsProtocolFeeControllerTest,
  OutOfBoundsProtocolFeeControllerTestInterface,
} from "../../ProtocolFeeControllerTest.sol/OutOfBoundsProtocolFeeControllerTest";

const _abi = [
  {
    type: "function",
    name: "protocolFeeForPool",
    inputs: [
      {
        name: "",
        type: "tuple",
        internalType: "struct PoolKey",
        components: [
          {
            name: "currency0",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "currency1",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "fee",
            type: "uint24",
            internalType: "uint24",
          },
          {
            name: "tickSpacing",
            type: "int24",
            internalType: "int24",
          },
          {
            name: "hooks",
            type: "address",
            internalType: "contract IHooks",
          },
        ],
      },
    ],
    outputs: [
      {
        name: "",
        type: "uint16",
        internalType: "uint16",
      },
    ],
    stateMutability: "pure",
  },
] as const;

const _bytecode =
  "0x608060405234801561001057600080fd5b5061017f806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c8063553bfc3714610030575b600080fd5b61004561003e3660046100a9565b5061100190565b60405161ffff909116815260200160405180910390f35b6001600160a01b038116811461007157600080fd5b50565b803561007f8161005c565b919050565b803562ffffff8116811461007f57600080fd5b8035600281900b811461007f57600080fd5b600060a082840312156100bb57600080fd5b60405160a0810181811067ffffffffffffffff821117156100ec57634e487b7160e01b600052604160045260246000fd5b60405282356100fa8161005c565b8152602083013561010a8161005c565b602082015261011b60408401610084565b604082015261012c60608401610097565b606082015261013d60808401610074565b6080820152939250505056fea2646970667358221220a93122ae864bc2705a10ad4a5941a4098a5212b32552c6c03bb711eefaa5796564736f6c63430008130033";

type OutOfBoundsProtocolFeeControllerTestConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: OutOfBoundsProtocolFeeControllerTestConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class OutOfBoundsProtocolFeeControllerTest__factory extends ContractFactory {
  constructor(...args: OutOfBoundsProtocolFeeControllerTestConstructorParams) {
    if (isSuperArgs(args)) {
      super(...args);
    } else {
      super(_abi, _bytecode, args[0]);
    }
  }

  override getDeployTransaction(
    overrides?: NonPayableOverrides & { from?: string }
  ): Promise<ContractDeployTransaction> {
    return super.getDeployTransaction(overrides || {});
  }
  override deploy(overrides?: NonPayableOverrides & { from?: string }) {
    return super.deploy(overrides || {}) as Promise<
      OutOfBoundsProtocolFeeControllerTest & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(
    runner: ContractRunner | null
  ): OutOfBoundsProtocolFeeControllerTest__factory {
    return super.connect(
      runner
    ) as OutOfBoundsProtocolFeeControllerTest__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): OutOfBoundsProtocolFeeControllerTestInterface {
    return new Interface(_abi) as OutOfBoundsProtocolFeeControllerTestInterface;
  }
  static connect(
    address: string,
    runner?: ContractRunner | null
  ): OutOfBoundsProtocolFeeControllerTest {
    return new Contract(
      address,
      _abi,
      runner
    ) as unknown as OutOfBoundsProtocolFeeControllerTest;
  }
}
