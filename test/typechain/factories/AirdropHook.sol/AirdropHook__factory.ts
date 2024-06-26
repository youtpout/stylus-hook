/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import {
  Contract,
  ContractFactory,
  ContractTransactionResponse,
  Interface,
} from "ethers";
import type {
  Signer,
  AddressLike,
  ContractDeployTransaction,
  ContractRunner,
} from "ethers";
import type { NonPayableOverrides } from "../../common";
import type {
  AirdropHook,
  AirdropHookInterface,
} from "../../AirdropHook.sol/AirdropHook";

const _abi = [
  {
    type: "constructor",
    inputs: [
      {
        name: "_poolManager",
        type: "address",
        internalType: "contract IPoolManager",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "afterAddLiquidity",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        name: "",
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
        name: "",
        type: "int256",
        internalType: "BalanceDelta",
      },
      {
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "afterDonate",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "afterInitialize",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        name: "",
        type: "uint160",
        internalType: "uint160",
      },
      {
        name: "",
        type: "int24",
        internalType: "int24",
      },
      {
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "afterRemoveLiquidity",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        name: "",
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
        name: "",
        type: "int256",
        internalType: "BalanceDelta",
      },
      {
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "afterSwap",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
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
        name: "swapParams",
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
        name: "",
        type: "int256",
        internalType: "BalanceDelta",
      },
      {
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "airdropToken",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "PoolId",
      },
    ],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "amountToClaim",
    inputs: [
      {
        name: "poolId",
        type: "bytes32",
        internalType: "PoolId",
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
    name: "beforeAddLiquidity",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        name: "",
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
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "beforeDonate",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "beforeInitialize",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        name: "",
        type: "uint160",
        internalType: "uint160",
      },
      {
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "beforeRemoveLiquidity",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        name: "",
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
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "beforeSwap",
    inputs: [
      {
        name: "",
        type: "address",
        internalType: "address",
      },
      {
        name: "",
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
        name: "",
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
        name: "",
        type: "bytes",
        internalType: "bytes",
      },
    ],
    outputs: [
      {
        name: "",
        type: "bytes4",
        internalType: "bytes4",
      },
    ],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "claimAirdrop",
    inputs: [
      {
        name: "poolId",
        type: "bytes32",
        internalType: "PoolId",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "claimed",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "PoolId",
      },
      {
        name: "",
        type: "address",
        internalType: "address",
      },
    ],
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
    name: "closeAirdrop",
    inputs: [
      {
        name: "poolId",
        type: "bytes32",
        internalType: "PoolId",
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
    name: "getHookPermissions",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "tuple",
        internalType: "struct Hooks.Permissions",
        components: [
          {
            name: "beforeInitialize",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "afterInitialize",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "beforeAddLiquidity",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "afterAddLiquidity",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "beforeRemoveLiquidity",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "afterRemoveLiquidity",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "beforeSwap",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "afterSwap",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "beforeDonate",
            type: "bool",
            internalType: "bool",
          },
          {
            name: "afterDonate",
            type: "bool",
            internalType: "bool",
          },
        ],
      },
    ],
    stateMutability: "pure",
  },
  {
    type: "function",
    name: "lockAcquired",
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
    name: "poolManager",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "address",
        internalType: "contract IPoolManager",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "totalSwap",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "PoolId",
      },
    ],
    outputs: [
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
        name: "counter0",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "counter1",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "totalSwapUser",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "PoolId",
      },
      {
        name: "",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [
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
        name: "counter0",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "counter1",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "totalUsers",
    inputs: [
      {
        name: "poolId",
        type: "bytes32",
        internalType: "PoolId",
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
    name: "userExist",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "PoolId",
      },
      {
        name: "",
        type: "address",
        internalType: "address",
      },
    ],
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
    name: "usersCount",
    inputs: [
      {
        name: "",
        type: "bytes32",
        internalType: "PoolId",
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
    name: "HookAddressNotValid",
    inputs: [
      {
        name: "hooks",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "HookNotImplemented",
    inputs: [],
  },
  {
    type: "error",
    name: "InvalidPool",
    inputs: [],
  },
  {
    type: "error",
    name: "LockFailure",
    inputs: [],
  },
  {
    type: "error",
    name: "NotPoolManager",
    inputs: [],
  },
  {
    type: "error",
    name: "NotSelf",
    inputs: [],
  },
] as const;

const _bytecode =
  "0x60a06040523480156200001157600080fd5b50604051620013fe380380620013fe83398101604081905262000034916200023a565b6001600160a01b038116608052806200004d3062000055565b50506200026c565b6200010a81620001046040805161014081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c0810182905260e081018290526101008101829052610120810191909152506040805161014081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c08101829052600160e0820152610100810182905261012081019190915290565b6200010d565b50565b805115156001609f1b83161515141580620001365750602081015115156001609e1b8316151514155b80620001505750604081015115156001609d1b8316151514155b806200016a5750606081015115156001609c1b8316151514155b80620001845750608081015115156001609b1b8316151514155b806200019e575060a081015115156001609a1b8316151514155b80620001b8575060c08101511515600160991b8316151514155b80620001d2575060e08101511515600160981b8316151514155b80620001ed57506101008101511515600160971b8316151514155b806200020857506101208101511515600160961b8316151514155b156200023657604051630732d7b560e51b81526001600160a01b038316600482015260240160405180910390fd5b5050565b6000602082840312156200024d57600080fd5b81516001600160a01b03811681146200026557600080fd5b9392505050565b60805161116f6200028f6000396000818161046d015261061c015261116f6000f3fe608060405234801561001057600080fd5b506004361061014d5760003560e01c8063a6ab2a43116100c3578063c3d5d4e61161007c578063c3d5d4e614610359578063c4e833ce14610375578063cfbe109b1461042b578063dc4c90d314610468578063dfcae6221461048f578063e1b4af691461036757600080fd5b8063a6ab2a4314610277578063a910f80f14610318578063ab6291fe14610326578063b47b2fb114610346578063b505b4ce14610359578063b6a8b0fa1461036757600080fd5b80632b58404e116101155780632b58404e146102775780632c4a8d64146102a35780633440d820146102b65780633afc005b146102c4578063575e24b414610277578063659db8ee1461030557600080fd5b80630d0f17041461015257806313c5a1891461018557806315f279f5146101a55780631aa42e2c146101e35780632326617114610221575b600080fd5b610172610160366004610b30565b60009081526002602052604090205490565b6040519081526020015b60405180910390f35b610172610193366004610b30565b60026020526000908152604090205481565b6101d36101b3366004610b71565b600360209081526000928352604080842090915290825290205460ff1681565b604051901515815260200161017c565b61021f6101f1366004610b71565b60009182526004602052604090912080546001600160a01b0319166001600160a01b03909216919091179055565b005b61025761022f366004610b30565b6001602081905260009182526040909120805491810154600282015460039092015490919084565b60408051948552602085019390935291830152606082015260800161017c565b61028a610285366004610c14565b6104bd565b6040516001600160e01b0319909116815260200161017c565b61021f6102b1366004610b30565b6104d8565b61028a610285366004610c90565b6102ed6102d2366004610b30565b6004602052600090815260409020546001600160a01b031681565b6040516001600160a01b03909116815260200161017c565b610172610313366004610b71565b6105e2565b61028a610285366004610d01565b610339610334366004610d8e565b61060f565b60405161017c9190610dd0565b61028a610354366004610e1e565b6106f7565b61028a610285366004610e1e565b61028a610285366004610e86565b61041e6040805161014081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c0810182905260e081018290526101008101829052610120810191909152506040805161014081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c08101829052600160e0820152610100810182905261012081019190915290565b60405161017c9190610ee5565b610257610439366004610b71565b600060208181529281526040808220909352908152208054600182015460028301546003909301549192909184565b6102ed7f000000000000000000000000000000000000000000000000000000000000000081565b6101d361049d366004610b71565b600560209081526000928352604080842090915290825290205460ff1681565b6000604051630a85dc2960e01b815260040160405180910390fd5b6000818152600460205260409020546001600160a01b03168061050e57604051630f06c3ed60e01b815260040160405180910390fd5b600082815260056020908152604080832033845290915290205460ff161561054957604051630c8d9eab60e31b815260040160405180910390fd5b60008281526005602090815260408083203380855292528220805460ff1916600117905561057a90849084906108fb565b604051635569f64b60e11b8152336004820152602481018290529091506001600160a01b0383169063aad3ec9690604401600060405180830381600087803b1580156105c557600080fd5b505af11580156105d9573d6000803e3d6000fd5b50505050505050565b6000828152600460205260408120546001600160a01b03166106058482856108fb565b9150505b92915050565b6060336001600160a01b037f0000000000000000000000000000000000000000000000000000000000000000161461065a5760405163570c108560e11b815260040160405180910390fd5b600080306001600160a01b03168585604051610677929190610f9d565b6000604051808303816000865af19150503d80600081146106b4576040519150601f19603f3d011682016040523d82523d6000602084013e6106b9565b606091505b509150915081156106cd5791506106099050565b80516000036106ef576040516314815f4760e31b815260040160405180910390fd5b805160208201fd5b6000328161071261070d368a90038a018a610fc0565b610a7a565b6000818152600460205260409020549091506001600160a01b031615610744575063b47b2fb160e01b91506108f19050565b60008181526003602090815260408083206001600160a01b038616845290915290205460ff166107b75760008181526003602090815260408083206001600160a01b03861684528252808320805460ff19166001179055838352600290915281208054916107b183611076565b91905055505b600080886020013513156107d0575060208701356107e0565b6107dd602089013561108f565b90505b6000828152602081815260408083206001600160a01b0387168452825280832085845260018352922090610816908b018b6110ab565b15610880578282600101600082825461082f91906110d4565b909155505060038201805490600061084683611076565b91905055508281600101600082825461085f91906110d4565b909155505060038101805490600061087683611076565b91905055506108e1565b8282600001600082825461089491906110d4565b90915550506002820180549060006108ab83611076565b9190505550828160000160008282546108c491906110d4565b90915550506002810180549060006108db83611076565b91905055505b5063b47b2fb160e01b9450505050505b9695505050505050565b600080836001600160a01b0316635ce97dbb6040518163ffffffff1660e01b8152600401602060405180830381865afa15801561093c573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061096091906110e7565b6000868152602081815260408083206001600160a01b038816845282528083208151608080820184528254825260018084015483870152600280850154848701526003948501546060808601919091528e895282885286892087519485018852805480865293810154988501989098529087015495830195909552949092015492820192909252815194955090939092916109fe9186916028610af0565b90506000610a1785856020015185602001516028610af0565b90506000610a308686604001518660400151600a610af0565b90506000610a498787606001518760600151600a610af0565b90508082610a5785876110d4565b610a6191906110d4565b610a6b91906110d4565b9b9a5050505050505050505050565b600081604051602001610ad3919081516001600160a01b03908116825260208084015182169083015260408084015162ffffff169083015260608084015160020b90830152608092830151169181019190915260a00190565b604051602081830303815290604052805190602001209050919050565b6000610afd836064611100565b610b089060016110d4565b82610b138688611100565b610b1d9190611100565b610b279190611117565b95945050505050565b600060208284031215610b4257600080fd5b5035919050565b6001600160a01b0381168114610b5e57600080fd5b50565b8035610b6c81610b49565b919050565b60008060408385031215610b8457600080fd5b823591506020830135610b9681610b49565b809150509250929050565b600060a08284031215610bb357600080fd5b50919050565b600060608284031215610bb357600080fd5b60008083601f840112610bdd57600080fd5b50813567ffffffffffffffff811115610bf557600080fd5b602083019150836020828501011115610c0d57600080fd5b9250929050565b60008060008060006101408688031215610c2d57600080fd5b8535610c3881610b49565b9450610c478760208801610ba1565b9350610c568760c08801610bb9565b925061012086013567ffffffffffffffff811115610c7357600080fd5b610c7f88828901610bcb565b969995985093965092949392505050565b60008060008060006101008688031215610ca957600080fd5b8535610cb481610b49565b9450610cc38760208801610ba1565b935060c0860135610cd381610b49565b925060e086013567ffffffffffffffff811115610c7357600080fd5b8035600281900b8114610b6c57600080fd5b6000806000806000806101208789031215610d1b57600080fd5b8635610d2681610b49565b9550610d358860208901610ba1565b945060c0870135610d4581610b49565b9350610d5360e08801610cef565b925061010087013567ffffffffffffffff811115610d7057600080fd5b610d7c89828a01610bcb565b979a9699509497509295939492505050565b60008060208385031215610da157600080fd5b823567ffffffffffffffff811115610db857600080fd5b610dc485828601610bcb565b90969095509350505050565b600060208083528351808285015260005b81811015610dfd57858101830151858201604001528201610de1565b506000604082860101526040601f19601f8301168501019250505092915050565b6000806000806000806101608789031215610e3857600080fd5b8635610e4381610b49565b9550610e528860208901610ba1565b9450610e618860c08901610bb9565b9350610120870135925061014087013567ffffffffffffffff811115610d7057600080fd5b6000806000806000806101208789031215610ea057600080fd5b8635610eab81610b49565b9550610eba8860208901610ba1565b945060c0870135935060e0870135925061010087013567ffffffffffffffff811115610d7057600080fd5b81511515815261014081016020830151610f03602084018215159052565b506040830151610f17604084018215159052565b506060830151610f2b606084018215159052565b506080830151610f3f608084018215159052565b5060a0830151610f5360a084018215159052565b5060c0830151610f6760c084018215159052565b5060e0830151610f7b60e084018215159052565b5061010083810151151590830152610120928301511515929091019190915290565b8183823760009101908152919050565b803562ffffff81168114610b6c57600080fd5b600060a08284031215610fd257600080fd5b60405160a0810181811067ffffffffffffffff8211171561100357634e487b7160e01b600052604160045260246000fd5b604052823561101181610b49565b8152602083013561102181610b49565b602082015261103260408401610fad565b604082015261104360608401610cef565b606082015261105460808401610b61565b60808201529392505050565b634e487b7160e01b600052601160045260246000fd5b60006001820161108857611088611060565b5060010190565b6000600160ff1b82016110a4576110a4611060565b5060000390565b6000602082840312156110bd57600080fd5b813580151581146110cd57600080fd5b9392505050565b8082018082111561060957610609611060565b6000602082840312156110f957600080fd5b5051919050565b808202811582820484141761060957610609611060565b60008261113457634e487b7160e01b600052601260045260246000fd5b50049056fea2646970667358221220b81e80fbb0b91524131d3558380f09c3ce466f6739ed2730698c6b95df427fec64736f6c63430008130033";

type AirdropHookConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: AirdropHookConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class AirdropHook__factory extends ContractFactory {
  constructor(...args: AirdropHookConstructorParams) {
    if (isSuperArgs(args)) {
      super(...args);
    } else {
      super(_abi, _bytecode, args[0]);
    }
  }

  override getDeployTransaction(
    _poolManager: AddressLike,
    overrides?: NonPayableOverrides & { from?: string }
  ): Promise<ContractDeployTransaction> {
    return super.getDeployTransaction(_poolManager, overrides || {});
  }
  override deploy(
    _poolManager: AddressLike,
    overrides?: NonPayableOverrides & { from?: string }
  ) {
    return super.deploy(_poolManager, overrides || {}) as Promise<
      AirdropHook & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(runner: ContractRunner | null): AirdropHook__factory {
    return super.connect(runner) as AirdropHook__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): AirdropHookInterface {
    return new Interface(_abi) as AirdropHookInterface;
  }
  static connect(address: string, runner?: ContractRunner | null): AirdropHook {
    return new Contract(address, _abi, runner) as unknown as AirdropHook;
  }
}
