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
  BigNumberish,
  AddressLike,
  ContractDeployTransaction,
  ContractRunner,
} from "ethers";
import type { NonPayableOverrides } from "../common";
import type { AirdropToken, AirdropTokenInterface } from "../AirdropToken";

const _abi = [
  {
    type: "constructor",
    inputs: [
      {
        name: "name",
        type: "string",
        internalType: "string",
      },
      {
        name: "symbol",
        type: "string",
        internalType: "string",
      },
      {
        name: "initialOwner",
        type: "address",
        internalType: "address",
      },
      {
        name: "_hook",
        type: "address",
        internalType: "address",
      },
      {
        name: "_amountAirDrop",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    stateMutability: "nonpayable",
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
    name: "approve",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "value",
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
        name: "account",
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
    name: "burn",
    inputs: [
      {
        name: "value",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "burnFrom",
    inputs: [
      {
        name: "account",
        type: "address",
        internalType: "address",
      },
      {
        name: "value",
        type: "uint256",
        internalType: "uint256",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "claim",
    inputs: [
      {
        name: "receiver",
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
    name: "decimals",
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
    name: "decreaseAllowance",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "requestedDecrease",
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
    name: "hook",
    inputs: [],
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
    name: "increaseAllowance",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "addedValue",
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
    name: "mint",
    inputs: [
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
    name: "name",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "owner",
    inputs: [],
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
    name: "renounceOwnership",
    inputs: [],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "restAirdrop",
    inputs: [],
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
    name: "symbol",
    inputs: [],
    outputs: [
      {
        name: "",
        type: "string",
        internalType: "string",
      },
    ],
    stateMutability: "view",
  },
  {
    type: "function",
    name: "totalAirdrop",
    inputs: [],
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
    name: "totalSupply",
    inputs: [],
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
    name: "transfer",
    inputs: [
      {
        name: "to",
        type: "address",
        internalType: "address",
      },
      {
        name: "value",
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
        name: "from",
        type: "address",
        internalType: "address",
      },
      {
        name: "to",
        type: "address",
        internalType: "address",
      },
      {
        name: "value",
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
    name: "transferOwnership",
    inputs: [
      {
        name: "newOwner",
        type: "address",
        internalType: "address",
      },
    ],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "event",
    name: "Approval",
    inputs: [
      {
        name: "owner",
        type: "address",
        indexed: true,
        internalType: "address",
      },
      {
        name: "spender",
        type: "address",
        indexed: true,
        internalType: "address",
      },
      {
        name: "value",
        type: "uint256",
        indexed: false,
        internalType: "uint256",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "OwnershipTransferred",
    inputs: [
      {
        name: "previousOwner",
        type: "address",
        indexed: true,
        internalType: "address",
      },
      {
        name: "newOwner",
        type: "address",
        indexed: true,
        internalType: "address",
      },
    ],
    anonymous: false,
  },
  {
    type: "event",
    name: "Transfer",
    inputs: [
      {
        name: "from",
        type: "address",
        indexed: true,
        internalType: "address",
      },
      {
        name: "to",
        type: "address",
        indexed: true,
        internalType: "address",
      },
      {
        name: "value",
        type: "uint256",
        indexed: false,
        internalType: "uint256",
      },
    ],
    anonymous: false,
  },
  {
    type: "error",
    name: "ERC20FailedDecreaseAllowance",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "currentAllowance",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "requestedDecrease",
        type: "uint256",
        internalType: "uint256",
      },
    ],
  },
  {
    type: "error",
    name: "ERC20InsufficientAllowance",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
      {
        name: "allowance",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "needed",
        type: "uint256",
        internalType: "uint256",
      },
    ],
  },
  {
    type: "error",
    name: "ERC20InsufficientBalance",
    inputs: [
      {
        name: "sender",
        type: "address",
        internalType: "address",
      },
      {
        name: "balance",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "needed",
        type: "uint256",
        internalType: "uint256",
      },
    ],
  },
  {
    type: "error",
    name: "ERC20InvalidApprover",
    inputs: [
      {
        name: "approver",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "ERC20InvalidReceiver",
    inputs: [
      {
        name: "receiver",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "ERC20InvalidSender",
    inputs: [
      {
        name: "sender",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "ERC20InvalidSpender",
    inputs: [
      {
        name: "spender",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "NotHook",
    inputs: [],
  },
  {
    type: "error",
    name: "OwnableInvalidOwner",
    inputs: [
      {
        name: "owner",
        type: "address",
        internalType: "address",
      },
    ],
  },
  {
    type: "error",
    name: "OwnableUnauthorizedAccount",
    inputs: [
      {
        name: "account",
        type: "address",
        internalType: "address",
      },
    ],
  },
] as const;

const _bytecode =
  "0x60c06040523480156200001157600080fd5b50604051620012353803806200123583398101604081905262000034916200035e565b848484808383600362000048838262000486565b50600462000057828262000486565b5050506200006b81620000b560201b60201c565b5062000094336200007f6012600a62000667565b6200008e90629896806200067f565b62000107565b50505060808190526006556001600160a01b031660a05250620006af915050565b600580546001600160a01b038381166001600160a01b0319831681179093556040519116919082907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a35050565b6001600160a01b038216620001375760405163ec442f0560e01b8152600060048201526024015b60405180910390fd5b620001456000838362000149565b5050565b6001600160a01b038316620001785780600260008282546200016c919062000699565b90915550620001ec9050565b6001600160a01b03831660009081526020819052604090205481811015620001cd5760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016200012e565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b0382166200020a5760028054829003905562000229565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516200026f91815260200190565b60405180910390a3505050565b634e487b7160e01b600052604160045260246000fd5b600082601f830112620002a457600080fd5b81516001600160401b0380821115620002c157620002c16200027c565b604051601f8301601f19908116603f01168101908282118183101715620002ec57620002ec6200027c565b816040528381526020925086838588010111156200030957600080fd5b600091505b838210156200032d57858201830151818301840152908201906200030e565b600093810190920192909252949350505050565b80516001600160a01b03811681146200035957600080fd5b919050565b600080600080600060a086880312156200037757600080fd5b85516001600160401b03808211156200038f57600080fd5b6200039d89838a0162000292565b96506020880151915080821115620003b457600080fd5b50620003c38882890162000292565b945050620003d46040870162000341565b9250620003e46060870162000341565b9150608086015190509295509295909350565b600181811c908216806200040c57607f821691505b6020821081036200042d57634e487b7160e01b600052602260045260246000fd5b50919050565b601f8211156200048157600081815260208120601f850160051c810160208610156200045c5750805b601f850160051c820191505b818110156200047d5782815560010162000468565b5050505b505050565b81516001600160401b03811115620004a257620004a26200027c565b620004ba81620004b38454620003f7565b8462000433565b602080601f831160018114620004f25760008415620004d95750858301515b600019600386901b1c1916600185901b1785556200047d565b600085815260208120601f198616915b82811015620005235788860151825594840194600190910190840162000502565b5085821015620005425787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b634e487b7160e01b600052601160045260246000fd5b600181815b80851115620005a95781600019048211156200058d576200058d62000552565b808516156200059b57918102915b93841c93908002906200056d565b509250929050565b600082620005c25750600162000661565b81620005d15750600062000661565b8160018114620005ea5760028114620005f55762000615565b600191505062000661565b60ff84111562000609576200060962000552565b50506001821b62000661565b5060208310610133831016604e8410600b84101617156200063a575081810a62000661565b62000646838362000568565b80600019048211156200065d576200065d62000552565b0290505b92915050565b60006200067860ff841683620005b1565b9392505050565b808202811582820484141762000661576200066162000552565b8082018082111562000661576200066162000552565b60805160a051610b59620006dc6000396000818161025c01526104d9015260006101f10152610b596000f3fe608060405234801561001057600080fd5b50600436106101375760003560e01c8063715018a6116100b8578063a457c2d71161007c578063a457c2d7146102af578063a9059cbb146102c2578063aad3ec96146102d5578063d519735b146102e8578063dd62ed3e146102f1578063f2fde38b1461030457600080fd5b8063715018a61461023c57806379cc6790146102445780637f5a7c7b146102575780638da5cb5b1461029657806395d89b41146102a757600080fd5b806339509351116100ff57806339509351146101b157806340c10f19146101c457806342966c68146101d95780635ce97dbb146101ec57806370a082311461021357600080fd5b806306fdde031461013c578063095ea7b31461015a57806318160ddd1461017d57806323b872dd1461018f578063313ce567146101a2575b600080fd5b610144610317565b604051610151919061096f565b60405180910390f35b61016d6101683660046109d9565b6103a9565b6040519015158152602001610151565b6002545b604051908152602001610151565b61016d61019d366004610a03565b6103c3565b60405160128152602001610151565b61016d6101bf3660046109d9565b6103e7565b6101d76101d23660046109d9565b610409565b005b6101d76101e7366004610a3f565b61041f565b6101817f000000000000000000000000000000000000000000000000000000000000000081565b610181610221366004610a58565b6001600160a01b031660009081526020819052604090205490565b6101d761042c565b6101d76102523660046109d9565b610440565b61027e7f000000000000000000000000000000000000000000000000000000000000000081565b6040516001600160a01b039091168152602001610151565b6005546001600160a01b031661027e565b610144610455565b61016d6102bd3660046109d9565b610464565b61016d6102d03660046109d9565b6104c0565b6101d76102e33660046109d9565b6104ce565b61018160065481565b6101816102ff366004610a7a565b610548565b6101d7610312366004610a58565b610573565b60606003805461032690610aad565b80601f016020809104026020016040519081016040528092919081815260200182805461035290610aad565b801561039f5780601f106103745761010080835404028352916020019161039f565b820191906000526020600020905b81548152906001019060200180831161038257829003601f168201915b5050505050905090565b6000336103b78185856105ae565b60019150505b92915050565b6000336103d18582856105c0565b6103dc858585610626565b506001949350505050565b6000336103b78185856103fa8383610548565b6104049190610afd565b6105ae565b610411610685565b61041b82826106b2565b5050565b61042933826106e8565b50565b610434610685565b61043e600061071e565b565b61044b8233836105c0565b61041b82826106e8565b60606004805461032690610aad565b600033816104728286610548565b9050838110156104b357604051632983c0c360e21b81526001600160a01b038616600482015260248101829052604481018590526064015b60405180910390fd5b6103dc82868684036105ae565b6000336103b7818585610626565b336001600160a01b037f00000000000000000000000000000000000000000000000000000000000000001614610517576040516318e59f8760e31b815260040160405180910390fd5b60065481111561052657506006545b80600660008282546105389190610b10565b9091555061041b905082826106b2565b6001600160a01b03918216600090815260016020908152604080832093909416825291909152205490565b61057b610685565b6001600160a01b0381166105a557604051631e4fbdf760e01b8152600060048201526024016104aa565b6104298161071e565b6105bb8383836001610770565b505050565b60006105cc8484610548565b90506000198114610620578181101561061157604051637dc7a0d960e11b81526001600160a01b038416600482015260248101829052604481018390526064016104aa565b61062084848484036000610770565b50505050565b6001600160a01b03831661065057604051634b637e8f60e11b8152600060048201526024016104aa565b6001600160a01b03821661067a5760405163ec442f0560e01b8152600060048201526024016104aa565b6105bb838383610845565b6005546001600160a01b0316331461043e5760405163118cdaa760e01b81523360048201526024016104aa565b6001600160a01b0382166106dc5760405163ec442f0560e01b8152600060048201526024016104aa565b61041b60008383610845565b6001600160a01b03821661071257604051634b637e8f60e11b8152600060048201526024016104aa565b61041b82600083610845565b600580546001600160a01b038381166001600160a01b0319831681179093556040519116919082907f8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e090600090a35050565b6001600160a01b03841661079a5760405163e602df0560e01b8152600060048201526024016104aa565b6001600160a01b0383166107c457604051634a1406b160e11b8152600060048201526024016104aa565b6001600160a01b038085166000908152600160209081526040808320938716835292905220829055801561062057826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161083791815260200190565b60405180910390a350505050565b6001600160a01b0383166108705780600260008282546108659190610afd565b909155506108e29050565b6001600160a01b038316600090815260208190526040902054818110156108c35760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016104aa565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b0382166108fe5760028054829003905561091d565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161096291815260200190565b60405180910390a3505050565b600060208083528351808285015260005b8181101561099c57858101830151858201604001528201610980565b506000604082860101526040601f19601f8301168501019250505092915050565b80356001600160a01b03811681146109d457600080fd5b919050565b600080604083850312156109ec57600080fd5b6109f5836109bd565b946020939093013593505050565b600080600060608486031215610a1857600080fd5b610a21846109bd565b9250610a2f602085016109bd565b9150604084013590509250925092565b600060208284031215610a5157600080fd5b5035919050565b600060208284031215610a6a57600080fd5b610a73826109bd565b9392505050565b60008060408385031215610a8d57600080fd5b610a96836109bd565b9150610aa4602084016109bd565b90509250929050565b600181811c90821680610ac157607f821691505b602082108103610ae157634e487b7160e01b600052602260045260246000fd5b50919050565b634e487b7160e01b600052601160045260246000fd5b808201808211156103bd576103bd610ae7565b818103818111156103bd576103bd610ae756fea26469706673582212206b5a01ac57243d4fb1a362d757242398060059c1730b3ce8c15cae9b67d4f8df64736f6c63430008130033";

type AirdropTokenConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: AirdropTokenConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class AirdropToken__factory extends ContractFactory {
  constructor(...args: AirdropTokenConstructorParams) {
    if (isSuperArgs(args)) {
      super(...args);
    } else {
      super(_abi, _bytecode, args[0]);
    }
  }

  override getDeployTransaction(
    name: string,
    symbol: string,
    initialOwner: AddressLike,
    _hook: AddressLike,
    _amountAirDrop: BigNumberish,
    overrides?: NonPayableOverrides & { from?: string }
  ): Promise<ContractDeployTransaction> {
    return super.getDeployTransaction(
      name,
      symbol,
      initialOwner,
      _hook,
      _amountAirDrop,
      overrides || {}
    );
  }
  override deploy(
    name: string,
    symbol: string,
    initialOwner: AddressLike,
    _hook: AddressLike,
    _amountAirDrop: BigNumberish,
    overrides?: NonPayableOverrides & { from?: string }
  ) {
    return super.deploy(
      name,
      symbol,
      initialOwner,
      _hook,
      _amountAirDrop,
      overrides || {}
    ) as Promise<
      AirdropToken & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(runner: ContractRunner | null): AirdropToken__factory {
    return super.connect(runner) as AirdropToken__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): AirdropTokenInterface {
    return new Interface(_abi) as AirdropTokenInterface;
  }
  static connect(
    address: string,
    runner?: ContractRunner | null
  ): AirdropToken {
    return new Contract(address, _abi, runner) as unknown as AirdropToken;
  }
}
