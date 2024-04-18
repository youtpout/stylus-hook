/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import {
  Contract,
  ContractFactory,
  ContractTransactionResponse,
  Interface,
} from "ethers";
import type { Signer, ContractDeployTransaction, ContractRunner } from "ethers";
import type { NonPayableOverrides } from "../../common";
import type { MockUNI, MockUNIInterface } from "../../mUNI.sol/MockUNI";

const _abi = [
  {
    type: "constructor",
    inputs: [],
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
] as const;

const _bytecode =
  "0x60806040523480156200001157600080fd5b50604051806040016040528060088152602001674d6f636b20554e4960c01b815250604051806040016040528060048152602001636d554e4960e01b8152508160039081620000619190620002d2565b506004620000708282620002d2565b505050620000ad3362000088620000b360201b60201c565b620000989060ff16600a620004b3565b620000a790620f4240620004c8565b620000b8565b620004f8565b601290565b6001600160a01b038216620000e85760405163ec442f0560e01b8152600060048201526024015b60405180910390fd5b620000f660008383620000fa565b5050565b6001600160a01b038316620001295780600260008282546200011d9190620004e2565b909155506200019d9050565b6001600160a01b038316600090815260208190526040902054818110156200017e5760405163391434e360e21b81526001600160a01b03851660048201526024810182905260448101839052606401620000df565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b038216620001bb57600280548290039055620001da565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516200022091815260200190565b60405180910390a3505050565b634e487b7160e01b600052604160045260246000fd5b600181811c908216806200025857607f821691505b6020821081036200027957634e487b7160e01b600052602260045260246000fd5b50919050565b601f821115620002cd57600081815260208120601f850160051c81016020861015620002a85750805b601f850160051c820191505b81811015620002c957828155600101620002b4565b5050505b505050565b81516001600160401b03811115620002ee57620002ee6200022d565b6200030681620002ff845462000243565b846200027f565b602080601f8311600181146200033e5760008415620003255750858301515b600019600386901b1c1916600185901b178555620002c9565b600085815260208120601f198616915b828110156200036f578886015182559484019460019091019084016200034e565b50858210156200038e5787850151600019600388901b60f8161c191681555b5050505050600190811b01905550565b634e487b7160e01b600052601160045260246000fd5b600181815b80851115620003f5578160001904821115620003d957620003d96200039e565b80851615620003e757918102915b93841c9390800290620003b9565b509250929050565b6000826200040e57506001620004ad565b816200041d57506000620004ad565b8160018114620004365760028114620004415762000461565b6001915050620004ad565b60ff8411156200045557620004556200039e565b50506001821b620004ad565b5060208310610133831016604e8410600b841016171562000486575081810a620004ad565b620004928383620003b4565b8060001904821115620004a957620004a96200039e565b0290505b92915050565b6000620004c18383620003fd565b9392505050565b8082028115828204841417620004ad57620004ad6200039e565b80820180821115620004ad57620004ad6200039e565b6107c280620005086000396000f3fe608060405234801561001057600080fd5b50600436106100a95760003560e01c80633950935111610071578063395093511461012357806370a082311461013657806395d89b411461015f578063a457c2d714610167578063a9059cbb1461017a578063dd62ed3e1461018d57600080fd5b806306fdde03146100ae578063095ea7b3146100cc57806318160ddd146100ef57806323b872dd14610101578063313ce56714610114575b600080fd5b6100b66101a0565b6040516100c3919061060c565b60405180910390f35b6100df6100da366004610676565b610232565b60405190151581526020016100c3565b6002545b6040519081526020016100c3565b6100df61010f3660046106a0565b61024c565b604051601281526020016100c3565b6100df610131366004610676565b610270565b6100f36101443660046106dc565b6001600160a01b031660009081526020819052604090205490565b6100b6610292565b6100df610175366004610676565b6102a1565b6100df610188366004610676565b6102fd565b6100f361019b3660046106fe565b61030b565b6060600380546101af90610731565b80601f01602080910402602001604051908101604052809291908181526020018280546101db90610731565b80156102285780601f106101fd57610100808354040283529160200191610228565b820191906000526020600020905b81548152906001019060200180831161020b57829003601f168201915b5050505050905090565b600033610240818585610336565b60019150505b92915050565b60003361025a858285610348565b6102658585856103ae565b506001949350505050565b600033610240818585610283838361030b565b61028d919061076b565b610336565b6060600480546101af90610731565b600033816102af828661030b565b9050838110156102f057604051632983c0c360e21b81526001600160a01b038616600482015260248101829052604481018590526064015b60405180910390fd5b6102658286868403610336565b6000336102408185856103ae565b6001600160a01b03918216600090815260016020908152604080832093909416825291909152205490565b610343838383600161040d565b505050565b6000610354848461030b565b905060001981146103a8578181101561039957604051637dc7a0d960e11b81526001600160a01b038416600482015260248101829052604481018390526064016102e7565b6103a88484848403600061040d565b50505050565b6001600160a01b0383166103d857604051634b637e8f60e11b8152600060048201526024016102e7565b6001600160a01b0382166104025760405163ec442f0560e01b8152600060048201526024016102e7565b6103438383836104e2565b6001600160a01b0384166104375760405163e602df0560e01b8152600060048201526024016102e7565b6001600160a01b03831661046157604051634a1406b160e11b8152600060048201526024016102e7565b6001600160a01b03808516600090815260016020908152604080832093871683529290522082905580156103a857826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040516104d491815260200190565b60405180910390a350505050565b6001600160a01b03831661050d578060026000828254610502919061076b565b9091555061057f9050565b6001600160a01b038316600090815260208190526040902054818110156105605760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016102e7565b6001600160a01b03841660009081526020819052604090209082900390555b6001600160a01b03821661059b576002805482900390556105ba565b6001600160a01b03821660009081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516105ff91815260200190565b60405180910390a3505050565b600060208083528351808285015260005b818110156106395785810183015185820160400152820161061d565b506000604082860101526040601f19601f8301168501019250505092915050565b80356001600160a01b038116811461067157600080fd5b919050565b6000806040838503121561068957600080fd5b6106928361065a565b946020939093013593505050565b6000806000606084860312156106b557600080fd5b6106be8461065a565b92506106cc6020850161065a565b9150604084013590509250925092565b6000602082840312156106ee57600080fd5b6106f78261065a565b9392505050565b6000806040838503121561071157600080fd5b61071a8361065a565b91506107286020840161065a565b90509250929050565b600181811c9082168061074557607f821691505b60208210810361076557634e487b7160e01b600052602260045260246000fd5b50919050565b8082018082111561024657634e487b7160e01b600052601160045260246000fdfea2646970667358221220c4645d6fbf74a19ba36c63cde0ca8b8ab014362baa9540a69dc2f8ca452080d664736f6c63430008130033";

type MockUNIConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: MockUNIConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class MockUNI__factory extends ContractFactory {
  constructor(...args: MockUNIConstructorParams) {
    if (isSuperArgs(args)) {
      super(...args);
    } else {
      super(_abi, _bytecode, args[0]);
    }
  }

  override getDeployTransaction(
    overrides?: NonPayableOverrides & { from?: string }
  ): Promise<ContractDeployTransaction> {
    return super.getDeployTransaction(overrides || {});
  }
  override deploy(overrides?: NonPayableOverrides & { from?: string }) {
    return super.deploy(overrides || {}) as Promise<
      MockUNI & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(runner: ContractRunner | null): MockUNI__factory {
    return super.connect(runner) as MockUNI__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): MockUNIInterface {
    return new Interface(_abi) as MockUNIInterface;
  }
  static connect(address: string, runner?: ContractRunner | null): MockUNI {
    return new Contract(address, _abi, runner) as unknown as MockUNI;
  }
}