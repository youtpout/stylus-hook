/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */

import { Contract, Interface, type ContractRunner } from "ethers";
import type { IAirdropHook, IAirdropHookInterface } from "../IAirdropHook";

const _abi = [
  {
    type: "function",
    name: "addAfterSwap",
    inputs: [
      {
        name: "pool_id",
        type: "bytes32",
        internalType: "bytes32",
      },
      {
        name: "sender",
        type: "address",
        internalType: "address",
      },
      {
        name: "zero_for_one",
        type: "bool",
        internalType: "bool",
      },
      {
        name: "amount_specified",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "amountToClaim",
    inputs: [
      {
        name: "pool_id",
        type: "bytes32",
        internalType: "bytes32",
      },
      {
        name: "receiver",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "amountToClaim",
    inputs: [
      {
        name: "pool_id",
        type: "bytes32",
        internalType: "bytes32",
      },
      {
        name: "token",
        type: "address",
        internalType: "address",
      },
      {
        name: "receiver",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "calculateTokenAirdrop",
    inputs: [
      {
        name: "amount_to_airdrop",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "user_volume",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "total_volume",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "percent",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "claim",
    inputs: [
      {
        name: "pool_id",
        type: "bytes32",
        internalType: "bytes32",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "claimAirdrop",
    inputs: [
      {
        name: "pool_id",
        type: "bytes32",
        internalType: "bytes32",
      },
      {
        name: "receiver",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "closeAirdrop",
    inputs: [
      {
        name: "pool_id",
        type: "bytes32",
        internalType: "bytes32",
      },
      {
        name: "token",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "getTotalSwap",
    inputs: [
      {
        name: "pool_id",
        type: "bytes32",
        internalType: "bytes32",
      },
    ],
    outputs: [
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "setHook",
    inputs: [
      {
        name: "value",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "error",
    name: "AirdropNotEnd",
    inputs: [],
  },
  {
    type: "error",
    name: "AlreadyClaimed",
    inputs: [],
  },
  {
    type: "error",
    name: "HookAlreadyDefined",
    inputs: [],
  },
  {
    type: "error",
    name: "NotHook",
    inputs: [],
  },
] as const;

export class IAirdropHook__factory {
  static readonly abi = _abi;
  static createInterface(): IAirdropHookInterface {
    return new Interface(_abi) as IAirdropHookInterface;
  }
  static connect(
    address: string,
    runner?: ContractRunner | null
  ): IAirdropHook {
    return new Contract(address, _abi, runner) as unknown as IAirdropHook;
  }
}
