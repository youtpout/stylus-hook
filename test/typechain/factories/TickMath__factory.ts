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
import type { TickMath, TickMathInterface } from "../TickMath";

const _abi = [
  {
    type: "error",
    name: "InvalidSqrtRatio",
    inputs: [],
  },
  {
    type: "error",
    name: "InvalidTick",
    inputs: [],
  },
] as const;

const _bytecode =
  "0x60566037600b82828239805160001a607314602a57634e487b7160e01b600052600060045260246000fd5b30600052607381538281f3fe73000000000000000000000000000000000000000030146080604052600080fdfea264697066735822122016e34819155731cdbf63793833bf1003e736f744b5cb7ea34cb8c4f181b0b31864736f6c63430008130033";

type TickMathConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: TickMathConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class TickMath__factory extends ContractFactory {
  constructor(...args: TickMathConstructorParams) {
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
      TickMath & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(runner: ContractRunner | null): TickMath__factory {
    return super.connect(runner) as TickMath__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): TickMathInterface {
    return new Interface(_abi) as TickMathInterface;
  }
  static connect(address: string, runner?: ContractRunner | null): TickMath {
    return new Contract(address, _abi, runner) as unknown as TickMath;
  }
}