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
import type { TickBitmap, TickBitmapInterface } from "../TickBitmap";

const _abi = [
  {
    type: "error",
    name: "TickMisaligned",
    inputs: [
      {
        name: "tick",
        type: "int24",
        internalType: "int24",
      },
      {
        name: "tickSpacing",
        type: "int24",
        internalType: "int24",
      },
    ],
  },
] as const;

const _bytecode =
  "0x60566037600b82828239805160001a607314602a57634e487b7160e01b600052600060045260246000fd5b30600052607381538281f3fe73000000000000000000000000000000000000000030146080604052600080fdfea264697066735822122067f9fd488c7c9ae33a45180b58b3ecd878a6ce1b8d6eedfa4ba59303e1b55a9c64736f6c63430008130033";

type TickBitmapConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: TickBitmapConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class TickBitmap__factory extends ContractFactory {
  constructor(...args: TickBitmapConstructorParams) {
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
      TickBitmap & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(runner: ContractRunner | null): TickBitmap__factory {
    return super.connect(runner) as TickBitmap__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): TickBitmapInterface {
    return new Interface(_abi) as TickBitmapInterface;
  }
  static connect(address: string, runner?: ContractRunner | null): TickBitmap {
    return new Contract(address, _abi, runner) as unknown as TickBitmap;
  }
}
