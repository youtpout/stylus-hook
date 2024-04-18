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
  OverflowProtocolFeeControllerTest,
  OverflowProtocolFeeControllerTestInterface,
} from "../../ProtocolFeeControllerTest.sol/OverflowProtocolFeeControllerTest";

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
  "0x608060405234801561001057600080fd5b5061018f806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c8063553bfc3714610030575b600080fd5b61004361003e3660046100b9565b61005a565b60405161ffff909116815260200160405180910390f35b600060405164ffffaaa0018152602081f35b6001600160a01b038116811461008157600080fd5b50565b803561008f8161006c565b919050565b803562ffffff8116811461008f57600080fd5b8035600281900b811461008f57600080fd5b600060a082840312156100cb57600080fd5b60405160a0810181811067ffffffffffffffff821117156100fc57634e487b7160e01b600052604160045260246000fd5b604052823561010a8161006c565b8152602083013561011a8161006c565b602082015261012b60408401610094565b604082015261013c606084016100a7565b606082015261014d60808401610084565b6080820152939250505056fea26469706673582212206a4d8754169d658cf9499d77daac572e7ff6fdb9d31e45af5674bec9a9b4fb4364736f6c63430008130033";

type OverflowProtocolFeeControllerTestConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: OverflowProtocolFeeControllerTestConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class OverflowProtocolFeeControllerTest__factory extends ContractFactory {
  constructor(...args: OverflowProtocolFeeControllerTestConstructorParams) {
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
      OverflowProtocolFeeControllerTest & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(
    runner: ContractRunner | null
  ): OverflowProtocolFeeControllerTest__factory {
    return super.connect(runner) as OverflowProtocolFeeControllerTest__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): OverflowProtocolFeeControllerTestInterface {
    return new Interface(_abi) as OverflowProtocolFeeControllerTestInterface;
  }
  static connect(
    address: string,
    runner?: ContractRunner | null
  ): OverflowProtocolFeeControllerTest {
    return new Contract(
      address,
      _abi,
      runner
    ) as unknown as OverflowProtocolFeeControllerTest;
  }
}