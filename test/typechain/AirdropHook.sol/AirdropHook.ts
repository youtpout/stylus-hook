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

export declare namespace Hooks {
  export type PermissionsStruct = {
    beforeInitialize: boolean;
    afterInitialize: boolean;
    beforeAddLiquidity: boolean;
    afterAddLiquidity: boolean;
    beforeRemoveLiquidity: boolean;
    afterRemoveLiquidity: boolean;
    beforeSwap: boolean;
    afterSwap: boolean;
    beforeDonate: boolean;
    afterDonate: boolean;
  };

  export type PermissionsStructOutput = [
    beforeInitialize: boolean,
    afterInitialize: boolean,
    beforeAddLiquidity: boolean,
    afterAddLiquidity: boolean,
    beforeRemoveLiquidity: boolean,
    afterRemoveLiquidity: boolean,
    beforeSwap: boolean,
    afterSwap: boolean,
    beforeDonate: boolean,
    afterDonate: boolean
  ] & {
    beforeInitialize: boolean;
    afterInitialize: boolean;
    beforeAddLiquidity: boolean;
    afterAddLiquidity: boolean;
    beforeRemoveLiquidity: boolean;
    afterRemoveLiquidity: boolean;
    beforeSwap: boolean;
    afterSwap: boolean;
    beforeDonate: boolean;
    afterDonate: boolean;
  };
}

export interface AirdropHookInterface extends Interface {
  getFunction(
    nameOrSignature:
      | "afterAddLiquidity"
      | "afterDonate"
      | "afterInitialize"
      | "afterRemoveLiquidity"
      | "afterSwap"
      | "airdropToken"
      | "amountToClaim"
      | "beforeAddLiquidity"
      | "beforeDonate"
      | "beforeInitialize"
      | "beforeRemoveLiquidity"
      | "beforeSwap"
      | "claimAirdrop"
      | "claimed"
      | "closeAirdrop"
      | "getHookPermissions"
      | "lockAcquired"
      | "poolManager"
      | "totalSwap"
      | "totalSwapUser"
      | "totalUsers"
      | "userExist"
      | "usersCount"
  ): FunctionFragment;

  encodeFunctionData(
    functionFragment: "afterAddLiquidity",
    values: [
      AddressLike,
      PoolKeyStruct,
      IPoolManager.ModifyLiquidityParamsStruct,
      BigNumberish,
      BytesLike
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "afterDonate",
    values: [AddressLike, PoolKeyStruct, BigNumberish, BigNumberish, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "afterInitialize",
    values: [AddressLike, PoolKeyStruct, BigNumberish, BigNumberish, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "afterRemoveLiquidity",
    values: [
      AddressLike,
      PoolKeyStruct,
      IPoolManager.ModifyLiquidityParamsStruct,
      BigNumberish,
      BytesLike
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "afterSwap",
    values: [
      AddressLike,
      PoolKeyStruct,
      IPoolManager.SwapParamsStruct,
      BigNumberish,
      BytesLike
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "airdropToken",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "amountToClaim",
    values: [BytesLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "beforeAddLiquidity",
    values: [
      AddressLike,
      PoolKeyStruct,
      IPoolManager.ModifyLiquidityParamsStruct,
      BytesLike
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "beforeDonate",
    values: [AddressLike, PoolKeyStruct, BigNumberish, BigNumberish, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "beforeInitialize",
    values: [AddressLike, PoolKeyStruct, BigNumberish, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "beforeRemoveLiquidity",
    values: [
      AddressLike,
      PoolKeyStruct,
      IPoolManager.ModifyLiquidityParamsStruct,
      BytesLike
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "beforeSwap",
    values: [
      AddressLike,
      PoolKeyStruct,
      IPoolManager.SwapParamsStruct,
      BytesLike
    ]
  ): string;
  encodeFunctionData(
    functionFragment: "claimAirdrop",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "claimed",
    values: [BytesLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "closeAirdrop",
    values: [BytesLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "getHookPermissions",
    values?: undefined
  ): string;
  encodeFunctionData(
    functionFragment: "lockAcquired",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "poolManager",
    values?: undefined
  ): string;
  encodeFunctionData(
    functionFragment: "totalSwap",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "totalSwapUser",
    values: [BytesLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "totalUsers",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "userExist",
    values: [BytesLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "usersCount",
    values: [BytesLike]
  ): string;

  decodeFunctionResult(
    functionFragment: "afterAddLiquidity",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "afterDonate",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "afterInitialize",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "afterRemoveLiquidity",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "afterSwap", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "airdropToken",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "amountToClaim",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "beforeAddLiquidity",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "beforeDonate",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "beforeInitialize",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "beforeRemoveLiquidity",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "beforeSwap", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "claimAirdrop",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "claimed", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "closeAirdrop",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "getHookPermissions",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "lockAcquired",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "poolManager",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "totalSwap", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "totalSwapUser",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "totalUsers", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "userExist", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "usersCount", data: BytesLike): Result;
}

export interface AirdropHook extends BaseContract {
  connect(runner?: ContractRunner | null): AirdropHook;
  waitForDeployment(): Promise<this>;

  interface: AirdropHookInterface;

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

  afterAddLiquidity: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  afterDonate: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  afterInitialize: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  afterRemoveLiquidity: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  afterSwap: TypedContractMethod<
    [
      arg0: AddressLike,
      key: PoolKeyStruct,
      swapParams: IPoolManager.SwapParamsStruct,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  airdropToken: TypedContractMethod<[arg0: BytesLike], [string], "view">;

  amountToClaim: TypedContractMethod<
    [poolId: BytesLike, receiver: AddressLike],
    [bigint],
    "view"
  >;

  beforeAddLiquidity: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  beforeDonate: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  beforeInitialize: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  beforeRemoveLiquidity: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  beforeSwap: TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.SwapParamsStruct,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;

  claimAirdrop: TypedContractMethod<[poolId: BytesLike], [void], "nonpayable">;

  claimed: TypedContractMethod<
    [arg0: BytesLike, arg1: AddressLike],
    [boolean],
    "view"
  >;

  closeAirdrop: TypedContractMethod<
    [poolId: BytesLike, token: AddressLike],
    [void],
    "nonpayable"
  >;

  getHookPermissions: TypedContractMethod<
    [],
    [Hooks.PermissionsStructOutput],
    "view"
  >;

  lockAcquired: TypedContractMethod<[data: BytesLike], [string], "nonpayable">;

  poolManager: TypedContractMethod<[], [string], "view">;

  totalSwap: TypedContractMethod<
    [arg0: BytesLike],
    [
      [bigint, bigint, bigint, bigint] & {
        amount0: bigint;
        amount1: bigint;
        counter0: bigint;
        counter1: bigint;
      }
    ],
    "view"
  >;

  totalSwapUser: TypedContractMethod<
    [arg0: BytesLike, arg1: AddressLike],
    [
      [bigint, bigint, bigint, bigint] & {
        amount0: bigint;
        amount1: bigint;
        counter0: bigint;
        counter1: bigint;
      }
    ],
    "view"
  >;

  totalUsers: TypedContractMethod<[poolId: BytesLike], [bigint], "view">;

  userExist: TypedContractMethod<
    [arg0: BytesLike, arg1: AddressLike],
    [boolean],
    "view"
  >;

  usersCount: TypedContractMethod<[arg0: BytesLike], [bigint], "view">;

  getFunction<T extends ContractMethod = ContractMethod>(
    key: string | FunctionFragment
  ): T;

  getFunction(
    nameOrSignature: "afterAddLiquidity"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "afterDonate"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "afterInitialize"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "afterRemoveLiquidity"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "afterSwap"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      key: PoolKeyStruct,
      swapParams: IPoolManager.SwapParamsStruct,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "airdropToken"
  ): TypedContractMethod<[arg0: BytesLike], [string], "view">;
  getFunction(
    nameOrSignature: "amountToClaim"
  ): TypedContractMethod<
    [poolId: BytesLike, receiver: AddressLike],
    [bigint],
    "view"
  >;
  getFunction(
    nameOrSignature: "beforeAddLiquidity"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "beforeDonate"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BigNumberish,
      arg4: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "beforeInitialize"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: BigNumberish,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "beforeRemoveLiquidity"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.ModifyLiquidityParamsStruct,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "beforeSwap"
  ): TypedContractMethod<
    [
      arg0: AddressLike,
      arg1: PoolKeyStruct,
      arg2: IPoolManager.SwapParamsStruct,
      arg3: BytesLike
    ],
    [string],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "claimAirdrop"
  ): TypedContractMethod<[poolId: BytesLike], [void], "nonpayable">;
  getFunction(
    nameOrSignature: "claimed"
  ): TypedContractMethod<
    [arg0: BytesLike, arg1: AddressLike],
    [boolean],
    "view"
  >;
  getFunction(
    nameOrSignature: "closeAirdrop"
  ): TypedContractMethod<
    [poolId: BytesLike, token: AddressLike],
    [void],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "getHookPermissions"
  ): TypedContractMethod<[], [Hooks.PermissionsStructOutput], "view">;
  getFunction(
    nameOrSignature: "lockAcquired"
  ): TypedContractMethod<[data: BytesLike], [string], "nonpayable">;
  getFunction(
    nameOrSignature: "poolManager"
  ): TypedContractMethod<[], [string], "view">;
  getFunction(
    nameOrSignature: "totalSwap"
  ): TypedContractMethod<
    [arg0: BytesLike],
    [
      [bigint, bigint, bigint, bigint] & {
        amount0: bigint;
        amount1: bigint;
        counter0: bigint;
        counter1: bigint;
      }
    ],
    "view"
  >;
  getFunction(
    nameOrSignature: "totalSwapUser"
  ): TypedContractMethod<
    [arg0: BytesLike, arg1: AddressLike],
    [
      [bigint, bigint, bigint, bigint] & {
        amount0: bigint;
        amount1: bigint;
        counter0: bigint;
        counter1: bigint;
      }
    ],
    "view"
  >;
  getFunction(
    nameOrSignature: "totalUsers"
  ): TypedContractMethod<[poolId: BytesLike], [bigint], "view">;
  getFunction(
    nameOrSignature: "userExist"
  ): TypedContractMethod<
    [arg0: BytesLike, arg1: AddressLike],
    [boolean],
    "view"
  >;
  getFunction(
    nameOrSignature: "usersCount"
  ): TypedContractMethod<[arg0: BytesLike], [bigint], "view">;

  filters: {};
}
