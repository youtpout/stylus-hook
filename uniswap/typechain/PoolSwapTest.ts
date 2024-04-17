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

export type PoolKeyStruct = {
  currency0: AddressLike;
  currency1: AddressLike;
  fee: BigNumberish;
  tickSpacing: BigNumberish;
  hooks: AddressLike;
};

export type PoolKeyStructOutput = [
  currency0: string,
  currency1: string,
  fee: bigint,
  tickSpacing: bigint,
  hooks: string
] & {
  currency0: string;
  currency1: string;
  fee: bigint;
  tickSpacing: bigint;
  hooks: string;
};

export declare namespace IPoolManager {
  export type SwapParamsStruct = {
    zeroForOne: boolean;
    amountSpecified: BigNumberish;
    sqrtPriceLimitX96: BigNumberish;
  };

  export type SwapParamsStructOutput = [
    zeroForOne: boolean,
    amountSpecified: bigint,
    sqrtPriceLimitX96: bigint
  ] & {
    zeroForOne: boolean;
    amountSpecified: bigint;
    sqrtPriceLimitX96: bigint;
  };
}

export declare namespace PoolSwapTest {
  export type TestSettingsStruct = {
    withdrawTokens: boolean;
    settleUsingTransfer: boolean;
    currencyAlreadySent: boolean;
  };

  export type TestSettingsStructOutput = [
    withdrawTokens: boolean,
    settleUsingTransfer: boolean,
    currencyAlreadySent: boolean
  ] & {
    withdrawTokens: boolean;
    settleUsingTransfer: boolean;
    currencyAlreadySent: boolean;
  };
}

export interface PoolSwapTestInterface extends Interface {
  getFunction(
    nameOrSignature: "manager" | "swap" | "unlockCallback"
  ): FunctionFragment;

  encodeFunctionData(functionFragment: "manager", values?: undefined): string;
  encodeFunctionData(
    functionFragment: "swap",
    values: [
      PoolKeyStruct,
      IPoolManager.SwapParamsStruct,
      PoolSwapTest.TestSettingsStruct,
      BytesLike
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "unlockCallback",
    values: [BytesLike]
  ): string;

  decodeFunctionResult(functionFragment: "manager", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "swap", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "unlockCallback",
    data: BytesLike
  ): Result;
}

export interface PoolSwapTest extends BaseContract {
  connect(runner?: ContractRunner | null): PoolSwapTest;
  waitForDeployment(): Promise<this>;

  interface: PoolSwapTestInterface;

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

  manager: TypedContractMethod<[], [string], "view">;

  swap: TypedContractMethod<
    [
      key: PoolKeyStruct,
      params: IPoolManager.SwapParamsStruct,
      testSettings: PoolSwapTest.TestSettingsStruct,
      hookData: BytesLike
    ],
    [bigint],
    "payable"
  >;

  unlockCallback: TypedContractMethod<
    [rawData: BytesLike],
    [string],
    "nonpayable"
  >;

  getFunction<T extends ContractMethod = ContractMethod>(
    key: string | FunctionFragment
  ): T;

  getFunction(
    nameOrSignature: "manager"
  ): TypedContractMethod<[], [string], "view">;
  getFunction(
    nameOrSignature: "swap"
  ): TypedContractMethod<
    [
      key: PoolKeyStruct,
      params: IPoolManager.SwapParamsStruct,
      testSettings: PoolSwapTest.TestSettingsStruct,
      hookData: BytesLike
    ],
    [bigint],
    "payable"
  >;
  getFunction(
    nameOrSignature: "unlockCallback"
  ): TypedContractMethod<[rawData: BytesLike], [string], "nonpayable">;

  filters: {};
}
