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
import type { NonPayableOverrides } from "../common";
import type { Pool, PoolInterface } from "../Pool";

const _abi = [
  {
    type: "error",
    name: "NoLiquidityToReceiveFees",
    inputs: [],
  },
  {
    type: "error",
    name: "PoolAlreadyInitialized",
    inputs: [],
  },
  {
    type: "error",
    name: "PoolNotInitialized",
    inputs: [],
  },
  {
    type: "error",
    name: "PriceLimitAlreadyExceeded",
    inputs: [
      {
        name: "sqrtPriceCurrentX96",
        type: "uint160",
        internalType: "uint160",
      },
      {
        name: "sqrtPriceLimitX96",
        type: "uint160",
        internalType: "uint160",
      },
    ],
  },
  {
    type: "error",
    name: "PriceLimitOutOfBounds",
    inputs: [
      {
        name: "sqrtPriceLimitX96",
        type: "uint160",
        internalType: "uint160",
      },
    ],
  },
  {
    type: "error",
    name: "SwapAmountCannotBeZero",
    inputs: [],
  },
  {
    type: "error",
    name: "TickLiquidityOverflow",
    inputs: [
      {
        name: "tick",
        type: "int24",
        internalType: "int24",
      },
    ],
  },
  {
    type: "error",
    name: "TickLowerOutOfBounds",
    inputs: [
      {
        name: "tickLower",
        type: "int24",
        internalType: "int24",
      },
    ],
  },
  {
    type: "error",
    name: "TickNotInitialized",
    inputs: [
      {
        name: "tick",
        type: "int24",
        internalType: "int24",
      },
    ],
  },
  {
    type: "error",
    name: "TickUpperOutOfBounds",
    inputs: [
      {
        name: "tickUpper",
        type: "int24",
        internalType: "int24",
      },
    ],
  },
  {
    type: "error",
    name: "TicksMisordered",
    inputs: [
      {
        name: "tickLower",
        type: "int24",
        internalType: "int24",
      },
      {
        name: "tickUpper",
        type: "int24",
        internalType: "int24",
      },
    ],
  },
] as const;

const _bytecode =
  "0x60566037600b82828239805160001a607314602a57634e487b7160e01b600052600060045260246000fd5b30600052607381538281f3fe73000000000000000000000000000000000000000030146080604052600080fdfea2646970667358221220a309013fcbc95aa18de958489837e26605ed28a764c614688fa7419e96bfcb7064736f6c63430008130033";

type PoolConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: PoolConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class Pool__factory extends ContractFactory {
  constructor(...args: PoolConstructorParams) {
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
      Pool & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(runner: ContractRunner | null): Pool__factory {
    return super.connect(runner) as Pool__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): PoolInterface {
    return new Interface(_abi) as PoolInterface;
  }
  static connect(address: string, runner?: ContractRunner | null): Pool {
    return new Contract(address, _abi, runner) as unknown as Pool;
  }
}