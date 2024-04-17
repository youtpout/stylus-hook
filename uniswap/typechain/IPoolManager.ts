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
  EventFragment,
  AddressLike,
  ContractRunner,
  ContractMethod,
  Listener,
} from "ethers";
import type {
  TypedContractEvent,
  TypedDeferredTopicFilter,
  TypedEventLog,
  TypedLogDescription,
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

export declare namespace Pool {
  export type TickInfoStruct = {
    liquidityGross: BigNumberish;
    liquidityNet: BigNumberish;
    feeGrowthOutside0X128: BigNumberish;
    feeGrowthOutside1X128: BigNumberish;
  };

  export type TickInfoStructOutput = [
    liquidityGross: bigint,
    liquidityNet: bigint,
    feeGrowthOutside0X128: bigint,
    feeGrowthOutside1X128: bigint
  ] & {
    liquidityGross: bigint;
    liquidityNet: bigint;
    feeGrowthOutside0X128: bigint;
    feeGrowthOutside1X128: bigint;
  };
}

export declare namespace Position {
  export type InfoStruct = {
    liquidity: BigNumberish;
    feeGrowthInside0LastX128: BigNumberish;
    feeGrowthInside1LastX128: BigNumberish;
  };

  export type InfoStructOutput = [
    liquidity: bigint,
    feeGrowthInside0LastX128: bigint,
    feeGrowthInside1LastX128: bigint
  ] & {
    liquidity: bigint;
    feeGrowthInside0LastX128: bigint;
    feeGrowthInside1LastX128: bigint;
  };
}

export declare namespace IPoolManager {
  export type ModifyLiquidityParamsStruct = {
    tickLower: BigNumberish;
    tickUpper: BigNumberish;
    liquidityDelta: BigNumberish;
  };

  export type ModifyLiquidityParamsStructOutput = [
    tickLower: bigint,
    tickUpper: bigint,
    liquidityDelta: bigint
  ] & { tickLower: bigint; tickUpper: bigint; liquidityDelta: bigint };

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

export interface IPoolManagerInterface extends Interface {
  getFunction(
    nameOrSignature:
      | "MAX_TICK_SPACING"
      | "MIN_PROTOCOL_FEE_DENOMINATOR"
      | "MIN_TICK_SPACING"
      | "allowance"
      | "approve"
      | "balanceOf"
      | "burn"
      | "currencyDelta"
      | "donate"
      | "extsload(bytes32)"
      | "extsload(bytes32,uint256)"
      | "getLiquidity(bytes32,address,int24,int24)"
      | "getLiquidity(bytes32)"
      | "getNonzeroDeltaCount"
      | "getPoolBitmapInfo"
      | "getPoolTickInfo"
      | "getPosition"
      | "getSlot0"
      | "initialize"
      | "isOperator"
      | "isUnlocked"
      | "mint"
      | "modifyLiquidity"
      | "protocolFeesAccrued"
      | "reservesOf"
      | "setOperator"
      | "setProtocolFee"
      | "settle"
      | "swap"
      | "take"
      | "transfer"
      | "transferFrom"
      | "unlock"
      | "updateDynamicSwapFee"
  ): FunctionFragment;

  getEvent(
    nameOrSignatureOrTopic:
      | "Initialize"
      | "ModifyLiquidity"
      | "ProtocolFeeControllerUpdated"
      | "ProtocolFeeUpdated"
      | "Swap"
  ): EventFragment;

  encodeFunctionData(
    functionFragment: "MAX_TICK_SPACING",
    values?: undefined
  ): string;
  encodeFunctionData(
    functionFragment: "MIN_PROTOCOL_FEE_DENOMINATOR",
    values?: undefined
  ): string;
  encodeFunctionData(
    functionFragment: "MIN_TICK_SPACING",
    values?: undefined
  ): string;
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
    functionFragment: "burn",
    values: [AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "currencyDelta",
    values: [AddressLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "donate",
    values: [PoolKeyStruct, BigNumberish, BigNumberish, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "extsload(bytes32)",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "extsload(bytes32,uint256)",
    values: [BytesLike, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "getLiquidity(bytes32,address,int24,int24)",
    values: [BytesLike, AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "getLiquidity(bytes32)",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "getNonzeroDeltaCount",
    values?: undefined
  ): string;
  encodeFunctionData(
    functionFragment: "getPoolBitmapInfo",
    values: [BytesLike, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "getPoolTickInfo",
    values: [BytesLike, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "getPosition",
    values: [BytesLike, AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(functionFragment: "getSlot0", values: [BytesLike]): string;
  encodeFunctionData(
    functionFragment: "initialize",
    values: [PoolKeyStruct, BigNumberish, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "isOperator",
    values: [AddressLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "isUnlocked",
    values?: undefined
  ): string;
  encodeFunctionData(
    functionFragment: "mint",
    values: [AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "modifyLiquidity",
    values: [PoolKeyStruct, IPoolManager.ModifyLiquidityParamsStruct, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "protocolFeesAccrued",
    values: [AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "reservesOf",
    values: [AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "setOperator",
    values: [AddressLike, boolean]
  ): string;
  encodeFunctionData(
    functionFragment: "setProtocolFee",
    values: [PoolKeyStruct]
  ): string;
  encodeFunctionData(functionFragment: "settle", values: [AddressLike]): string;
  encodeFunctionData(
    functionFragment: "swap",
    values: [PoolKeyStruct, IPoolManager.SwapParamsStruct, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "take",
    values: [AddressLike, AddressLike, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "transfer",
    values: [AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(
    functionFragment: "transferFrom",
    values: [AddressLike, AddressLike, BigNumberish, BigNumberish]
  ): string;
  encodeFunctionData(functionFragment: "unlock", values: [BytesLike]): string;
  encodeFunctionData(
    functionFragment: "updateDynamicSwapFee",
    values: [PoolKeyStruct, BigNumberish]
  ): string;

  decodeFunctionResult(
    functionFragment: "MAX_TICK_SPACING",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "MIN_PROTOCOL_FEE_DENOMINATOR",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "MIN_TICK_SPACING",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "allowance", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "approve", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "balanceOf", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "burn", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "currencyDelta",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "donate", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "extsload(bytes32)",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "extsload(bytes32,uint256)",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getLiquidity(bytes32,address,int24,int24)",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getLiquidity(bytes32)",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getNonzeroDeltaCount",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getPoolBitmapInfo",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getPoolTickInfo",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getPosition",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "getSlot0", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "initialize", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "isOperator", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "isUnlocked", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "mint", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "modifyLiquidity",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "protocolFeesAccrued",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "reservesOf", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "setOperator",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "setProtocolFee",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "settle", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "swap", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "take", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "transfer", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "transferFrom",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "unlock", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "updateDynamicSwapFee",
    data: BytesLike
  ): Result;
}

export namespace InitializeEvent {
  export type InputTuple = [
    id: BytesLike,
    currency0: AddressLike,
    currency1: AddressLike,
    fee: BigNumberish,
    tickSpacing: BigNumberish,
    hooks: AddressLike
  ];
  export type OutputTuple = [
    id: string,
    currency0: string,
    currency1: string,
    fee: bigint,
    tickSpacing: bigint,
    hooks: string
  ];
  export interface OutputObject {
    id: string;
    currency0: string;
    currency1: string;
    fee: bigint;
    tickSpacing: bigint;
    hooks: string;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export namespace ModifyLiquidityEvent {
  export type InputTuple = [
    id: BytesLike,
    sender: AddressLike,
    tickLower: BigNumberish,
    tickUpper: BigNumberish,
    liquidityDelta: BigNumberish
  ];
  export type OutputTuple = [
    id: string,
    sender: string,
    tickLower: bigint,
    tickUpper: bigint,
    liquidityDelta: bigint
  ];
  export interface OutputObject {
    id: string;
    sender: string;
    tickLower: bigint;
    tickUpper: bigint;
    liquidityDelta: bigint;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export namespace ProtocolFeeControllerUpdatedEvent {
  export type InputTuple = [protocolFeeController: AddressLike];
  export type OutputTuple = [protocolFeeController: string];
  export interface OutputObject {
    protocolFeeController: string;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export namespace ProtocolFeeUpdatedEvent {
  export type InputTuple = [id: BytesLike, protocolFee: BigNumberish];
  export type OutputTuple = [id: string, protocolFee: bigint];
  export interface OutputObject {
    id: string;
    protocolFee: bigint;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export namespace SwapEvent {
  export type InputTuple = [
    id: BytesLike,
    sender: AddressLike,
    amount0: BigNumberish,
    amount1: BigNumberish,
    sqrtPriceX96: BigNumberish,
    liquidity: BigNumberish,
    tick: BigNumberish,
    fee: BigNumberish
  ];
  export type OutputTuple = [
    id: string,
    sender: string,
    amount0: bigint,
    amount1: bigint,
    sqrtPriceX96: bigint,
    liquidity: bigint,
    tick: bigint,
    fee: bigint
  ];
  export interface OutputObject {
    id: string;
    sender: string;
    amount0: bigint;
    amount1: bigint;
    sqrtPriceX96: bigint;
    liquidity: bigint;
    tick: bigint;
    fee: bigint;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export interface IPoolManager extends BaseContract {
  connect(runner?: ContractRunner | null): IPoolManager;
  waitForDeployment(): Promise<this>;

  interface: IPoolManagerInterface;

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

  MAX_TICK_SPACING: TypedContractMethod<[], [bigint], "view">;

  MIN_PROTOCOL_FEE_DENOMINATOR: TypedContractMethod<[], [bigint], "view">;

  MIN_TICK_SPACING: TypedContractMethod<[], [bigint], "view">;

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

  burn: TypedContractMethod<
    [from: AddressLike, id: BigNumberish, amount: BigNumberish],
    [void],
    "nonpayable"
  >;

  currencyDelta: TypedContractMethod<
    [caller: AddressLike, currency: AddressLike],
    [bigint],
    "view"
  >;

  donate: TypedContractMethod<
    [
      key: PoolKeyStruct,
      amount0: BigNumberish,
      amount1: BigNumberish,
      hookData: BytesLike
    ],
    [bigint],
    "nonpayable"
  >;

  "extsload(bytes32)": TypedContractMethod<[slot: BytesLike], [string], "view">;

  "extsload(bytes32,uint256)": TypedContractMethod<
    [slot: BytesLike, nSlots: BigNumberish],
    [string],
    "view"
  >;

  "getLiquidity(bytes32,address,int24,int24)": TypedContractMethod<
    [
      id: BytesLike,
      owner: AddressLike,
      tickLower: BigNumberish,
      tickUpper: BigNumberish
    ],
    [bigint],
    "view"
  >;

  "getLiquidity(bytes32)": TypedContractMethod<
    [id: BytesLike],
    [bigint],
    "view"
  >;

  getNonzeroDeltaCount: TypedContractMethod<[], [bigint], "view">;

  getPoolBitmapInfo: TypedContractMethod<
    [id: BytesLike, word: BigNumberish],
    [bigint],
    "view"
  >;

  getPoolTickInfo: TypedContractMethod<
    [id: BytesLike, tick: BigNumberish],
    [Pool.TickInfoStructOutput],
    "view"
  >;

  getPosition: TypedContractMethod<
    [
      id: BytesLike,
      owner: AddressLike,
      tickLower: BigNumberish,
      tickUpper: BigNumberish
    ],
    [Position.InfoStructOutput],
    "view"
  >;

  getSlot0: TypedContractMethod<
    [id: BytesLike],
    [
      [bigint, bigint, bigint, bigint] & {
        sqrtPriceX96: bigint;
        tick: bigint;
        protocolFee: bigint;
        swapFee: bigint;
      }
    ],
    "view"
  >;

  initialize: TypedContractMethod<
    [key: PoolKeyStruct, sqrtPriceX96: BigNumberish, hookData: BytesLike],
    [bigint],
    "nonpayable"
  >;

  isOperator: TypedContractMethod<
    [owner: AddressLike, spender: AddressLike],
    [boolean],
    "view"
  >;

  isUnlocked: TypedContractMethod<[], [boolean], "view">;

  mint: TypedContractMethod<
    [to: AddressLike, id: BigNumberish, amount: BigNumberish],
    [void],
    "nonpayable"
  >;

  modifyLiquidity: TypedContractMethod<
    [
      key: PoolKeyStruct,
      params: IPoolManager.ModifyLiquidityParamsStruct,
      hookData: BytesLike
    ],
    [bigint],
    "nonpayable"
  >;

  protocolFeesAccrued: TypedContractMethod<
    [arg0: AddressLike],
    [bigint],
    "view"
  >;

  reservesOf: TypedContractMethod<[currency: AddressLike], [bigint], "view">;

  setOperator: TypedContractMethod<
    [spender: AddressLike, approved: boolean],
    [boolean],
    "nonpayable"
  >;

  setProtocolFee: TypedContractMethod<
    [key: PoolKeyStruct],
    [void],
    "nonpayable"
  >;

  settle: TypedContractMethod<[token: AddressLike], [bigint], "payable">;

  swap: TypedContractMethod<
    [
      key: PoolKeyStruct,
      params: IPoolManager.SwapParamsStruct,
      hookData: BytesLike
    ],
    [bigint],
    "nonpayable"
  >;

  take: TypedContractMethod<
    [currency: AddressLike, to: AddressLike, amount: BigNumberish],
    [void],
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

  unlock: TypedContractMethod<[data: BytesLike], [string], "nonpayable">;

  updateDynamicSwapFee: TypedContractMethod<
    [key: PoolKeyStruct, newDynamicSwapFee: BigNumberish],
    [void],
    "nonpayable"
  >;

  getFunction<T extends ContractMethod = ContractMethod>(
    key: string | FunctionFragment
  ): T;

  getFunction(
    nameOrSignature: "MAX_TICK_SPACING"
  ): TypedContractMethod<[], [bigint], "view">;
  getFunction(
    nameOrSignature: "MIN_PROTOCOL_FEE_DENOMINATOR"
  ): TypedContractMethod<[], [bigint], "view">;
  getFunction(
    nameOrSignature: "MIN_TICK_SPACING"
  ): TypedContractMethod<[], [bigint], "view">;
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
    nameOrSignature: "burn"
  ): TypedContractMethod<
    [from: AddressLike, id: BigNumberish, amount: BigNumberish],
    [void],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "currencyDelta"
  ): TypedContractMethod<
    [caller: AddressLike, currency: AddressLike],
    [bigint],
    "view"
  >;
  getFunction(
    nameOrSignature: "donate"
  ): TypedContractMethod<
    [
      key: PoolKeyStruct,
      amount0: BigNumberish,
      amount1: BigNumberish,
      hookData: BytesLike
    ],
    [bigint],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "extsload(bytes32)"
  ): TypedContractMethod<[slot: BytesLike], [string], "view">;
  getFunction(
    nameOrSignature: "extsload(bytes32,uint256)"
  ): TypedContractMethod<
    [slot: BytesLike, nSlots: BigNumberish],
    [string],
    "view"
  >;
  getFunction(
    nameOrSignature: "getLiquidity(bytes32,address,int24,int24)"
  ): TypedContractMethod<
    [
      id: BytesLike,
      owner: AddressLike,
      tickLower: BigNumberish,
      tickUpper: BigNumberish
    ],
    [bigint],
    "view"
  >;
  getFunction(
    nameOrSignature: "getLiquidity(bytes32)"
  ): TypedContractMethod<[id: BytesLike], [bigint], "view">;
  getFunction(
    nameOrSignature: "getNonzeroDeltaCount"
  ): TypedContractMethod<[], [bigint], "view">;
  getFunction(
    nameOrSignature: "getPoolBitmapInfo"
  ): TypedContractMethod<[id: BytesLike, word: BigNumberish], [bigint], "view">;
  getFunction(
    nameOrSignature: "getPoolTickInfo"
  ): TypedContractMethod<
    [id: BytesLike, tick: BigNumberish],
    [Pool.TickInfoStructOutput],
    "view"
  >;
  getFunction(
    nameOrSignature: "getPosition"
  ): TypedContractMethod<
    [
      id: BytesLike,
      owner: AddressLike,
      tickLower: BigNumberish,
      tickUpper: BigNumberish
    ],
    [Position.InfoStructOutput],
    "view"
  >;
  getFunction(
    nameOrSignature: "getSlot0"
  ): TypedContractMethod<
    [id: BytesLike],
    [
      [bigint, bigint, bigint, bigint] & {
        sqrtPriceX96: bigint;
        tick: bigint;
        protocolFee: bigint;
        swapFee: bigint;
      }
    ],
    "view"
  >;
  getFunction(
    nameOrSignature: "initialize"
  ): TypedContractMethod<
    [key: PoolKeyStruct, sqrtPriceX96: BigNumberish, hookData: BytesLike],
    [bigint],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "isOperator"
  ): TypedContractMethod<
    [owner: AddressLike, spender: AddressLike],
    [boolean],
    "view"
  >;
  getFunction(
    nameOrSignature: "isUnlocked"
  ): TypedContractMethod<[], [boolean], "view">;
  getFunction(
    nameOrSignature: "mint"
  ): TypedContractMethod<
    [to: AddressLike, id: BigNumberish, amount: BigNumberish],
    [void],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "modifyLiquidity"
  ): TypedContractMethod<
    [
      key: PoolKeyStruct,
      params: IPoolManager.ModifyLiquidityParamsStruct,
      hookData: BytesLike
    ],
    [bigint],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "protocolFeesAccrued"
  ): TypedContractMethod<[arg0: AddressLike], [bigint], "view">;
  getFunction(
    nameOrSignature: "reservesOf"
  ): TypedContractMethod<[currency: AddressLike], [bigint], "view">;
  getFunction(
    nameOrSignature: "setOperator"
  ): TypedContractMethod<
    [spender: AddressLike, approved: boolean],
    [boolean],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "setProtocolFee"
  ): TypedContractMethod<[key: PoolKeyStruct], [void], "nonpayable">;
  getFunction(
    nameOrSignature: "settle"
  ): TypedContractMethod<[token: AddressLike], [bigint], "payable">;
  getFunction(
    nameOrSignature: "swap"
  ): TypedContractMethod<
    [
      key: PoolKeyStruct,
      params: IPoolManager.SwapParamsStruct,
      hookData: BytesLike
    ],
    [bigint],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "take"
  ): TypedContractMethod<
    [currency: AddressLike, to: AddressLike, amount: BigNumberish],
    [void],
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
  getFunction(
    nameOrSignature: "unlock"
  ): TypedContractMethod<[data: BytesLike], [string], "nonpayable">;
  getFunction(
    nameOrSignature: "updateDynamicSwapFee"
  ): TypedContractMethod<
    [key: PoolKeyStruct, newDynamicSwapFee: BigNumberish],
    [void],
    "nonpayable"
  >;

  getEvent(
    key: "Initialize"
  ): TypedContractEvent<
    InitializeEvent.InputTuple,
    InitializeEvent.OutputTuple,
    InitializeEvent.OutputObject
  >;
  getEvent(
    key: "ModifyLiquidity"
  ): TypedContractEvent<
    ModifyLiquidityEvent.InputTuple,
    ModifyLiquidityEvent.OutputTuple,
    ModifyLiquidityEvent.OutputObject
  >;
  getEvent(
    key: "ProtocolFeeControllerUpdated"
  ): TypedContractEvent<
    ProtocolFeeControllerUpdatedEvent.InputTuple,
    ProtocolFeeControllerUpdatedEvent.OutputTuple,
    ProtocolFeeControllerUpdatedEvent.OutputObject
  >;
  getEvent(
    key: "ProtocolFeeUpdated"
  ): TypedContractEvent<
    ProtocolFeeUpdatedEvent.InputTuple,
    ProtocolFeeUpdatedEvent.OutputTuple,
    ProtocolFeeUpdatedEvent.OutputObject
  >;
  getEvent(
    key: "Swap"
  ): TypedContractEvent<
    SwapEvent.InputTuple,
    SwapEvent.OutputTuple,
    SwapEvent.OutputObject
  >;

  filters: {
    "Initialize(bytes32,address,address,uint24,int24,address)": TypedContractEvent<
      InitializeEvent.InputTuple,
      InitializeEvent.OutputTuple,
      InitializeEvent.OutputObject
    >;
    Initialize: TypedContractEvent<
      InitializeEvent.InputTuple,
      InitializeEvent.OutputTuple,
      InitializeEvent.OutputObject
    >;

    "ModifyLiquidity(bytes32,address,int24,int24,int256)": TypedContractEvent<
      ModifyLiquidityEvent.InputTuple,
      ModifyLiquidityEvent.OutputTuple,
      ModifyLiquidityEvent.OutputObject
    >;
    ModifyLiquidity: TypedContractEvent<
      ModifyLiquidityEvent.InputTuple,
      ModifyLiquidityEvent.OutputTuple,
      ModifyLiquidityEvent.OutputObject
    >;

    "ProtocolFeeControllerUpdated(address)": TypedContractEvent<
      ProtocolFeeControllerUpdatedEvent.InputTuple,
      ProtocolFeeControllerUpdatedEvent.OutputTuple,
      ProtocolFeeControllerUpdatedEvent.OutputObject
    >;
    ProtocolFeeControllerUpdated: TypedContractEvent<
      ProtocolFeeControllerUpdatedEvent.InputTuple,
      ProtocolFeeControllerUpdatedEvent.OutputTuple,
      ProtocolFeeControllerUpdatedEvent.OutputObject
    >;

    "ProtocolFeeUpdated(bytes32,uint16)": TypedContractEvent<
      ProtocolFeeUpdatedEvent.InputTuple,
      ProtocolFeeUpdatedEvent.OutputTuple,
      ProtocolFeeUpdatedEvent.OutputObject
    >;
    ProtocolFeeUpdated: TypedContractEvent<
      ProtocolFeeUpdatedEvent.InputTuple,
      ProtocolFeeUpdatedEvent.OutputTuple,
      ProtocolFeeUpdatedEvent.OutputObject
    >;

    "Swap(bytes32,address,int128,int128,uint160,uint128,int24,uint24)": TypedContractEvent<
      SwapEvent.InputTuple,
      SwapEvent.OutputTuple,
      SwapEvent.OutputObject
    >;
    Swap: TypedContractEvent<
      SwapEvent.InputTuple,
      SwapEvent.OutputTuple,
      SwapEvent.OutputObject
    >;
  };
}
