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
} from "../common";

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

export interface ProtocolFeeControllerTestInterface extends Interface {
  getFunction(
    nameOrSignature:
      | "protocolFeeForPool"
      | "setSwapFeeForPool"
      | "swapFeeForPool"
  ): FunctionFragment;

  encodeFunctionData(
    functionFragment: "protocolFeeForPool",
    values: [PoolKeyStruct]
  ): string;
  encodeFunctionData(
    functionFragment: "setSwapFeeForPool",
    values: [BytesLike, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "swapFeeForPool",
    values: [BytesLike]
  ): string;

  decodeFunctionResult(
    functionFragment: "protocolFeeForPool",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "setSwapFeeForPool",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "swapFeeForPool",
    data: BytesLike
  ): Result;
}

export interface ProtocolFeeControllerTest extends BaseContract {
  connect(runner?: ContractRunner | null): ProtocolFeeControllerTest;
  waitForDeployment(): Promise<this>;

  interface: ProtocolFeeControllerTestInterface;

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

  protocolFeeForPool: TypedContractMethod<
    [key: PoolKeyStruct],
    [bigint],
    "view"
  >;

  setSwapFeeForPool: TypedContractMethod<
    [id: BytesLike, fee: BigNumberish],
    [void],
    "nonpayable"
  >;

  swapFeeForPool: TypedContractMethod<[arg0: BytesLike], [bigint], "view">;

  getFunction<T extends ContractMethod = ContractMethod>(
    key: string | FunctionFragment
  ): T;

  getFunction(
    nameOrSignature: "protocolFeeForPool"
  ): TypedContractMethod<[key: PoolKeyStruct], [bigint], "view">;
  getFunction(
    nameOrSignature: "setSwapFeeForPool"
  ): TypedContractMethod<
    [id: BytesLike, fee: BigNumberish],
    [void],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "swapFeeForPool"
  ): TypedContractMethod<[arg0: BytesLike], [bigint], "view">;

  filters: {};
}
