/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import type {
  BaseContract,
  BigNumberish,
  BytesLike,
  FunctionFragment,
  Result,
  Interface,
  AddressLike,
  ContractRunner,
  ContractMethod,
  Listener,
} from "ethers";
import type {
  TypedContractEvent,
  TypedDeferredTopicFilter,
  TypedEventLog,
  TypedListener,
  TypedContractMethod,
} from "./common";

export interface IERC6909ClaimsInterface extends Interface {
  getFunction(
    nameOrSignature:
      | "allowance"
      | "approve"
      | "balanceOf"
      | "isOperator"
      | "setOperator"
      | "transfer"
      | "transferFrom"
  ): FunctionFragment;

  encodeFunctionData(
    functionFragment: "allowance",
    values: [AddressLike, AddressLike, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "approve",
    values: [AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "balanceOf",
    values: [AddressLike, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "isOperator",
    values: [AddressLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "setOperator",
    values: [AddressLike, boolean]
  ): string;
  encodeFunctionData(
    functionFragment: "transfer",
    values: [AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "transferFrom",
    values: [AddressLike, AddressLike, BigNumberish, BigNumberish]
  ): string;

  decodeFunctionResult(functionFragment: "allowance", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "approve", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "balanceOf", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "isOperator", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "setOperator",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "transfer", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "transferFrom",
    data: BytesLike
  ): Result;
}

export interface IERC6909Claims extends BaseContract {
  connect(runner?: ContractRunner | null): IERC6909Claims;
  waitForDeployment(): Promise<this>;

  interface: IERC6909ClaimsInterface;

  queryFilter<TCEvent extends TypedContractEvent>(
    event: TCEvent,
    fromBlockOrBlockhash?: string | number | undefined,
    toBlock?: string | number | undefined
  ): Promise<Array<TypedEventLog<TCEvent>>>;
  queryFilter<TCEvent extends TypedContractEvent>(
    filter: TypedDeferredTopicFilter<TCEvent>,
    fromBlockOrBlockhash?: string | number | undefined,
    toBlock?: string | number | undefined
  ): Promise<Array<TypedEventLog<TCEvent>>>;

  on<TCEvent extends TypedContractEvent>(
    event: TCEvent,
    listener: TypedListener<TCEvent>
  ): Promise<this>;
  on<TCEvent extends TypedContractEvent>(
    filter: TypedDeferredTopicFilter<TCEvent>,
    listener: TypedListener<TCEvent>
  ): Promise<this>;

  once<TCEvent extends TypedContractEvent>(
    event: TCEvent,
    listener: TypedListener<TCEvent>
  ): Promise<this>;
  once<TCEvent extends TypedContractEvent>(
    filter: TypedDeferredTopicFilter<TCEvent>,
    listener: TypedListener<TCEvent>
  ): Promise<this>;

  listeners<TCEvent extends TypedContractEvent>(
    event: TCEvent
  ): Promise<Array<TypedListener<TCEvent>>>;
  listeners(eventName?: string): Promise<Array<Listener>>;
  removeAllListeners<TCEvent extends TypedContractEvent>(
    event?: TCEvent
  ): Promise<this>;

  allowance: TypedContractMethod<
    [owner: AddressLike, spender: AddressLike, id: BigNumberish],
    [bigint],
    "view"
  >;

  approve: TypedContractMethod<
    [spender: AddressLike, id: BigNumberish, amount: BigNumberish],
    [boolean],
    "nonpayable"
  >;

  balanceOf: TypedContractMethod<
    [owner: AddressLike, id: BigNumberish],
    [bigint],
    "view"
  >;

  isOperator: TypedContractMethod<
    [owner: AddressLike, spender: AddressLike],
    [boolean],
    "view"
  >;

  setOperator: TypedContractMethod<
    [spender: AddressLike, approved: boolean],
    [boolean],
    "nonpayable"
  >;

  transfer: TypedContractMethod<
    [receiver: AddressLike, id: BigNumberish, amount: BigNumberish],
    [boolean],
    "nonpayable"
  >;

  transferFrom: TypedContractMethod<
    [
      sender: AddressLike,
      receiver: AddressLike,
      id: BigNumberish,
      amount: BigNumberish
    ],
    [boolean],
    "nonpayable"
  >;

  getFunction<T extends ContractMethod = ContractMethod>(
    key: string | FunctionFragment
  ): T;

  getFunction(
    nameOrSignature: "allowance"
  ): TypedContractMethod<
    [owner: AddressLike, spender: AddressLike, id: BigNumberish],
    [bigint],
    "view"
  >;
  getFunction(
    nameOrSignature: "approve"
  ): TypedContractMethod<
    [spender: AddressLike, id: BigNumberish, amount: BigNumberish],
    [boolean],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "balanceOf"
  ): TypedContractMethod<
    [owner: AddressLike, id: BigNumberish],
    [bigint],
    "view"
  >;
  getFunction(
    nameOrSignature: "isOperator"
  ): TypedContractMethod<
    [owner: AddressLike, spender: AddressLike],
    [boolean],
    "view"
  >;
  getFunction(
    nameOrSignature: "setOperator"
  ): TypedContractMethod<
    [spender: AddressLike, approved: boolean],
    [boolean],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "transfer"
  ): TypedContractMethod<
    [receiver: AddressLike, id: BigNumberish, amount: BigNumberish],
    [boolean],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "transferFrom"
  ): TypedContractMethod<
    [
      sender: AddressLike,
      receiver: AddressLike,
      id: BigNumberish,
      amount: BigNumberish
    ],
    [boolean],
    "nonpayable"
  >;

  filters: {};
}
