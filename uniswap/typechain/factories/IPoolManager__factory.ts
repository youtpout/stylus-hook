/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */

import { Contract, Interface, type ContractRunner } from "ethers";
import type { IPoolManager, IPoolManagerInterface } from "../IPoolManager";

const _abi = [
  {
    type: "function",
    name: "MAX_TICK_SPACING",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "int24",
        internalType: "int24",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "MIN_PROTOCOL_FEE_DENOMINATOR",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "uint8",
        internalType: "uint8",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "MIN_TICK_SPACING",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "int24",
        internalType: "int24",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "allowance",
    inputs: [
      {
        name: "owner",
        type: "address",
        internalType: "address",
      },
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "id",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "approve",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "id",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bool",
        internalType: "bool",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "balanceOf",
    inputs: [
      {
        name: "owner",
        type: "address",
        internalType: "address",
      },
      {
        name: "id",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "burn",
    inputs: [
      {
        name: "from",
        type: "address",
        internalType: "address",
      },
      {
        name: "id",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "currencyDelta",
    inputs: [
      {
        name: "caller",
        type: "address",
        internalType: "address",
      },
      {
        name: "currency",
        type: "address",
        internalType: "Currency",
      },
    ],
    outputs: [
      {
        name: "",
        type: "int256",
        internalType: "int256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "donate",
    inputs: [
      {
        name: "key",
        type: "tuple",
        internalType: "struct PoolKey",
        components: [
          {
            name: "currency0",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "currency1",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "fee",
            type: "uint24",
            internalType: "uint24",
          },
          {
            name: "tickSpacing",
            type: "int24",
            internalType: "int24",
          },
          {
            name: "hooks",
            type: "address",
            internalType: "contract IHooks",
          },
        ],
      },
      {
        name: "amount0",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "amount1",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "hookData",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "int256",
        internalType: "BalanceDelta",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "extsload",
    inputs: [
      {
        name: "slot",
        type: "bytes32",
        internalType: "bytes32",
      },
    ],
    outputs: [
      {
        name: "value",
        type: "bytes32",
        internalType: "bytes32",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "extsload",
    inputs: [
      {
        name: "slot",
        type: "bytes32",
        internalType: "bytes32",
      },
      {
        name: "nSlots",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [
      {
        name: "value",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getLiquidity",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        internalType: "PoolId",
      },
      {
        name: "owner",
        type: "address",
        internalType: "address",
      },
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
    outputs: [
      {
        name: "liquidity",
        type: "uint128",
        internalType: "uint128",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getLiquidity",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        internalType: "PoolId",
      },
    ],
    outputs: [
      {
        name: "liquidity",
        type: "uint128",
        internalType: "uint128",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getNonzeroDeltaCount",
    inputs: [],
    outputs: [
      {
        name: "_nonzeroDeltaCount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getPoolBitmapInfo",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        internalType: "PoolId",
      },
      {
        name: "word",
        type: "int16",
        internalType: "int16",
      },
    ],
    outputs: [
      {
        name: "tickBitmap",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getPoolTickInfo",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        internalType: "PoolId",
      },
      {
        name: "tick",
        type: "int24",
        internalType: "int24",
      },
    ],
    outputs: [
      {
        name: "",
        type: "tuple",
        internalType: "struct Pool.TickInfo",
        components: [
          {
            name: "liquidityGross",
            type: "uint128",
            internalType: "uint128",
          },
          {
            name: "liquidityNet",
            type: "int128",
            internalType: "int128",
          },
          {
            name: "feeGrowthOutside0X128",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "feeGrowthOutside1X128",
            type: "uint256",
            internalType: "uint256",
          },
        ],
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getPosition",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        internalType: "PoolId",
      },
      {
        name: "owner",
        type: "address",
        internalType: "address",
      },
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
    outputs: [
      {
        name: "position",
        type: "tuple",
        internalType: "struct Position.Info",
        components: [
          {
            name: "liquidity",
            type: "uint128",
            internalType: "uint128",
          },
          {
            name: "feeGrowthInside0LastX128",
            type: "uint256",
            internalType: "uint256",
          },
          {
            name: "feeGrowthInside1LastX128",
            type: "uint256",
            internalType: "uint256",
          },
        ],
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "getSlot0",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        internalType: "PoolId",
      },
    ],
    outputs: [
      {
        name: "sqrtPriceX96",
        type: "uint160",
        internalType: "uint160",
      },
      {
        name: "tick",
        type: "int24",
        internalType: "int24",
      },
      {
        name: "protocolFee",
        type: "uint16",
        internalType: "uint16",
      },
      {
        name: "swapFee",
        type: "uint24",
        internalType: "uint24",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "initialize",
    inputs: [
      {
        name: "key",
        type: "tuple",
        internalType: "struct PoolKey",
        components: [
          {
            name: "currency0",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "currency1",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "fee",
            type: "uint24",
            internalType: "uint24",
          },
          {
            name: "tickSpacing",
            type: "int24",
            internalType: "int24",
          },
          {
            name: "hooks",
            type: "address",
            internalType: "contract IHooks",
          },
        ],
      },
      {
        name: "sqrtPriceX96",
        type: "uint160",
        internalType: "uint160",
      },
      {
        name: "hookData",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "tick",
        type: "int24",
        internalType: "int24",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "isOperator",
    inputs: [
      {
        name: "owner",
        type: "address",
        internalType: "address",
      },
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [
      {
        name: "approved",
        type: "bool",
        internalType: "bool",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "isUnlocked",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "bool",
        internalType: "bool",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "mint",
    inputs: [
      {
        name: "to",
        type: "address",
        internalType: "address",
      },
      {
        name: "id",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "modifyLiquidity",
    inputs: [
      {
        name: "key",
        type: "tuple",
        internalType: "struct PoolKey",
        components: [
          {
            name: "currency0",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "currency1",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "fee",
            type: "uint24",
            internalType: "uint24",
          },
          {
            name: "tickSpacing",
            type: "int24",
            internalType: "int24",
          },
          {
            name: "hooks",
            type: "address",
            internalType: "contract IHooks",
          },
        ],
      },
      {
        name: "params",
        type: "tuple",
        internalType: "struct IPoolManager.ModifyLiquidityParams",
        components: [
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
          {
            name: "liquidityDelta",
            type: "int256",
            internalType: "int256",
          },
        ],
      },
      {
        name: "hookData",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "int256",
        internalType: "BalanceDelta",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "protocolFeesAccrued",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "Currency",
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
    name: "reservesOf",
    inputs: [
      {
        name: "currency",
        type: "address",
        internalType: "Currency",
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
    name: "setOperator",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "approved",
        type: "bool",
        internalType: "bool",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bool",
        internalType: "bool",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "setProtocolFee",
    inputs: [
      {
        name: "key",
        type: "tuple",
        internalType: "struct PoolKey",
        components: [
          {
            name: "currency0",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "currency1",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "fee",
            type: "uint24",
            internalType: "uint24",
          },
          {
            name: "tickSpacing",
            type: "int24",
            internalType: "int24",
          },
          {
            name: "hooks",
            type: "address",
            internalType: "contract IHooks",
          },
        ],
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "settle",
    inputs: [
      {
        name: "token",
        type: "address",
        internalType: "Currency",
      },
    ],
    outputs: [
      {
        name: "paid",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "payable",
  },
  {
    type: "function",
    name: "swap",
    inputs: [
      {
        name: "key",
        type: "tuple",
        internalType: "struct PoolKey",
        components: [
          {
            name: "currency0",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "currency1",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "fee",
            type: "uint24",
            internalType: "uint24",
          },
          {
            name: "tickSpacing",
            type: "int24",
            internalType: "int24",
          },
          {
            name: "hooks",
            type: "address",
            internalType: "contract IHooks",
          },
        ],
      },
      {
        name: "params",
        type: "tuple",
        internalType: "struct IPoolManager.SwapParams",
        components: [
          {
            name: "zeroForOne",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "amountSpecified",
            type: "int256",
            internalType: "int256",
          },
          {
            name: "sqrtPriceLimitX96",
            type: "uint160",
            internalType: "uint160",
          },
        ],
      },
      {
        name: "hookData",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "int256",
        internalType: "BalanceDelta",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "take",
    inputs: [
      {
        name: "currency",
        type: "address",
        internalType: "Currency",
      },
      {
        name: "to",
        type: "address",
        internalType: "address",
      },
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "transfer",
    inputs: [
      {
        name: "receiver",
        type: "address",
        internalType: "address",
      },
      {
        name: "id",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bool",
        internalType: "bool",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "transferFrom",
    inputs: [
      {
        name: "sender",
        type: "address",
        internalType: "address",
      },
      {
        name: "receiver",
        type: "address",
        internalType: "address",
      },
      {
        name: "id",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "amount",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bool",
        internalType: "bool",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "unlock",
    inputs: [
      {
        name: "data",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "updateDynamicSwapFee",
    inputs: [
      {
        name: "key",
        type: "tuple",
        internalType: "struct PoolKey",
        components: [
          {
            name: "currency0",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "currency1",
            type: "address",
            internalType: "Currency",
          },
          {
            name: "fee",
            type: "uint24",
            internalType: "uint24",
          },
          {
            name: "tickSpacing",
            type: "int24",
            internalType: "int24",
          },
          {
            name: "hooks",
            type: "address",
            internalType: "contract IHooks",
          },
        ],
      },
      {
        name: "newDynamicSwapFee",
        type: "uint24",
        internalType: "uint24",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "event",
    name: "Initialize",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        indexed: true,
        internalType: "PoolId",
      },
      {
        name: "currency0",
        type: "address",
        indexed: true,
        internalType: "Currency",
      },
      {
        name: "currency1",
        type: "address",
        indexed: true,
        internalType: "Currency",
      },
      {
        name: "fee",
        type: "uint24",
        indexed: false,
        internalType: "uint24",
      },
      {
        name: "tickSpacing",
        type: "int24",
        indexed: false,
        internalType: "int24",
      },
      {
        name: "hooks",
        type: "address",
        indexed: false,
        internalType: "contract IHooks",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "ModifyLiquidity",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        indexed: true,
        internalType: "PoolId",
      },
      {
        name: "sender",
        type: "address",
        indexed: true,
        internalType: "address",
      },
      {
        name: "tickLower",
        type: "int24",
        indexed: false,
        internalType: "int24",
      },
      {
        name: "tickUpper",
        type: "int24",
        indexed: false,
        internalType: "int24",
      },
      {
        name: "liquidityDelta",
        type: "int256",
        indexed: false,
        internalType: "int256",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "ProtocolFeeControllerUpdated",
    inputs: [
      {
        name: "protocolFeeController",
        type: "address",
        indexed: false,
        internalType: "address",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "ProtocolFeeUpdated",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        indexed: true,
        internalType: "PoolId",
      },
      {
        name: "protocolFee",
        type: "uint16",
        indexed: false,
        internalType: "uint16",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "Swap",
    inputs: [
      {
        name: "id",
        type: "bytes32",
        indexed: true,
        internalType: "PoolId",
      },
      {
        name: "sender",
        type: "address",
        indexed: true,
        internalType: "address",
      },
      {
        name: "amount0",
        type: "int128",
        indexed: false,
        internalType: "int128",
      },
      {
        name: "amount1",
        type: "int128",
        indexed: false,
        internalType: "int128",
      },
      {
        name: "sqrtPriceX96",
        type: "uint160",
        indexed: false,
        internalType: "uint160",
      },
      {
        name: "liquidity",
        type: "uint128",
        indexed: false,
        internalType: "uint128",
      },
      {
        name: "tick",
        type: "int24",
        indexed: false,
        internalType: "int24",
      },
      {
        name: "fee",
        type: "uint24",
        indexed: false,
        internalType: "uint24",
      },
    ],
    anonymous: false,
  },
  {
    type: "error",
    name: "AlreadyUnlocked",
    inputs: [],
  },
  {
    type: "error",
    name: "CurrenciesOutOfOrderOrEqual",
    inputs: [],
  },
  {
    type: "error",
    name: "CurrencyNotSettled",
    inputs: [],
  },
  {
    type: "error",
    name: "FeeNotDynamic",
    inputs: [],
  },
  {
    type: "error",
    name: "ManagerLocked",
    inputs: [],
  },
  {
    type: "error",
    name: "PoolNotInitialized",
    inputs: [],
  },
  {
    type: "error",
    name: "ProtocolFeeCannotBeFetched",
    inputs: [],
  },
  {
    type: "error",
    name: "ProtocolFeeControllerCallFailedOrInvalidResult",
    inputs: [],
  },
  {
    type: "error",
    name: "TickSpacingTooLarge",
    inputs: [],
  },
  {
    type: "error",
    name: "TickSpacingTooSmall",
    inputs: [],
  },
  {
    type: "error",
    name: "UnauthorizedDynamicSwapFeeUpdate",
    inputs: [],
  },
] as const;

export class IPoolManager__factory {
  static readonly abi = _abi;
  static createInterface(): IPoolManagerInterface {
    return new Interface(_abi) as IPoolManagerInterface;
  }
  static connect(
    address: string,
    runner?: ContractRunner | null
  ): IPoolManager {
    return new Contract(address, _abi, runner) as unknown as IPoolManager;
  }
}
