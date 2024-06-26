/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */

import { Contract, Interface, type ContractRunner } from "ethers";
import type {
  NoDelegateCall,
  NoDelegateCallInterface,
} from "../NoDelegateCall";

const _abi = [
  {
    type: "error",
    name: "DelegateCallNotAllowed",
    inputs: [],
  },
] as const;

export class NoDelegateCall__factory {
  static readonly abi = _abi;
  static createInterface(): NoDelegateCallInterface {
    return new Interface(_abi) as NoDelegateCallInterface;
  }
  static connect(
    address: string,
    runner?: ContractRunner | null
  ): NoDelegateCall {
    return new Contract(address, _abi, runner) as unknown as NoDelegateCall;
  }
}
