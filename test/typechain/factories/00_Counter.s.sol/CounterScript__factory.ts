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
import type {
  CounterScript,
  CounterScriptInterface,
} from "../../00_Counter.s.sol/CounterScript";

const _abi = [
  {
    type: "function",
    name: "IS_SCRIPT",
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
    name: "run",
    inputs: [],
    outputs: [],
    stateMutability: "nonpayable",
  },
  {
    type: "function",
    name: "setUp",
    inputs: [],
    outputs: [],
    stateMutability: "nonpayable",
  },
] as const;

const _bytecode =
  "0x608060405260048054600160ff199182168117909255600c8054909116909117905534801561002d57600080fd5b506112098061003d6000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c80630a9254e414610046578063c040622614610048578063f8ccbf4714610050575b600080fd5b005b610046610071565b600c5461005d9060ff1681565b604051901515815260200160405180910390f35b604051602b60981b9060009081906100eb90734e59b44847b379578588920ca78fbf26c0b4956c9085906100a760208201610363565b601f1982820381018352601f909101166040818152733a9d48ab9751398bbfa63ad67599bb04e4bdf98b602083015201604051602081830303815290604052610234565b915091507f885cb69240a935d632d79c317109709ecfa91a80626ff3989d68f67f5b1dd12d60001c6001600160a01b031663afc980406040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561014d57600080fd5b505af1158015610161573d6000803e3d6000fd5b50505050600081733a9d48ab9751398bbfa63ad67599bb04e4bdf98b60405161018990610363565b6001600160a01b0390911681526020018190604051809103906000f59050801580156101b9573d6000803e3d6000fd5b509050826001600160a01b0316816001600160a01b03161461022e5760405162461bcd60e51b8152602060048201526024808201527f436f756e7465725363726970743a20686f6f6b2061646472657373206d69736d6044820152630c2e8c6d60e31b60648201526084015b60405180910390fd5b50505050565b600080600080858560405160200161024d9291906103a0565b604051602081830303815290604052905060005b62030d40811015610312578151602080840191909120604080516001600160f81b0319818501526bffffffffffffffffffffffff1960608e901b1660218201526035810185905260558082019390935281518082039093018352607501905280519101209250610fff60941b83166001600160a01b0389161480156102ee57506001600160a01b0383163b155b156103005791935090915061035a9050565b8061030a816103bd565b915050610261565b60405162461bcd60e51b815260206004820152601e60248201527f486f6f6b4d696e65723a20636f756c64206e6f742066696e642073616c7400006044820152606401610225565b94509492505050565b610def806103e583390190565b6000815160005b818110156103915760208185018101518683015201610377565b50600093019283525090919050565b60006103b56103af8386610370565b84610370565b949350505050565b6000600182016103dd57634e487b7160e01b600052601160045260246000fd5b506001019056fe60a060405234801561001057600080fd5b5060405162000def38038062000def8339810160408190526100319161022a565b6001600160a01b038116608052806100483061004f565b505061025a565b610104816100ff6040805161014081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c0810182905260e0810182905261010081018290526101208101919091525060408051610140810182526000808252602082018190526001928201839052606082018190526080820183905260a0820181905260c0820183905260e0820192909252610100810182905261012081019190915290565b610107565b50565b805115156001609f1b8316151514158061012f5750602081015115156001609e1b8316151514155b806101485750604081015115156001609d1b8316151514155b806101615750606081015115156001609c1b8316151514155b8061017a5750608081015115156001609b1b8316151514155b80610193575060a081015115156001609a1b8316151514155b806101ac575060c08101511515600160991b8316151514155b806101c5575060e08101511515600160981b8316151514155b806101df57506101008101511515600160971b8316151514155b806101f957506101208101511515600160961b8316151514155b1561022657604051630732d7b560e51b81526001600160a01b038316600482015260240160405180910390fd5b5050565b60006020828403121561023c57600080fd5b81516001600160a01b038116811461025357600080fd5b9392505050565b608051610b726200027d6000396000818161032301526104700152610b726000f3fe608060405234801561001057600080fd5b506004361061010b5760003560e01c8063b505b4ce116100a2578063c693995711610071578063c6939957146102be578063dacdbe77146102de578063db6a9451146102fe578063dc4c90d31461031e578063e1b4af69146101f757600080fd5b8063b505b4ce146101e9578063b6a8b0fa146101f7578063c3d5d4e6146101e9578063c4e833ce1461020557600080fd5b8063a6ab2a43116100de578063a6ab2a4314610195578063a910f80f146101a8578063ab6291fe146101b6578063b47b2fb1146101d657600080fd5b80632b58404e146101105780633440d82014610141578063575e24b4146101545780635c338d3214610167575b600080fd5b61012361011e3660046106b0565b61035d565b6040516001600160e01b031990911681526020015b60405180910390f35b61012361014f36600461072c565b6103af565b6101236101623660046106b0565b6103ca565b61018761017536600461078b565b60006020819052908152604090205481565b604051908152602001610138565b6101236101a33660046106b0565b610416565b61012361014f3660046107b6565b6101c96101c4366004610843565b610463565b6040516101389190610885565b6101236101e43660046108d3565b610551565b61012361014f3660046108d3565b61012361014f36600461093b565b6102b16040805161014081018252600080825260208201819052918101829052606081018290526080810182905260a0810182905260c0810182905260e0810182905261010081018290526101208101919091525060408051610140810182526000808252602082018190526001928201839052606082018190526080820183905260a0820181905260c0820183905260e0820192909252610100810182905261012081019190915290565b604051610138919061099a565b6101876102cc36600461078b565b60026020526000908152604090205481565b6101876102ec36600461078b565b60016020526000908152604090205481565b61018761030c36600461078b565b60036020526000908152604090205481565b6103457f000000000000000000000000000000000000000000000000000000000000000081565b6040516001600160a01b039091168152602001610138565b600060038161037961037436899003890189610a65565b61059f565b8152602001908152602001600020600081548092919061039890610b05565b909155506315ac202760e11b979650505050505050565b6000604051630a85dc2960e01b815260040160405180910390fd5b600080806103e061037436899003890189610a65565b815260200190815260200160002060008154809291906103ff90610b05565b909155506315d7892d60e21b979650505050505050565b600060028161042d61037436899003890189610a65565b8152602001908152602001600020600081548092919061044c90610b05565b9091555063a6ab2a4360e01b979650505050505050565b6060336001600160a01b037f000000000000000000000000000000000000000000000000000000000000000016146104ae5760405163570c108560e11b815260040160405180910390fd5b600080306001600160a01b031685856040516104cb929190610b2c565b6000604051808303816000865af19150503d8060008114610508576040519150601f19603f3d011682016040523d82523d6000602084013e61050d565b606091505b5091509150811561052157915061054b9050565b8051600003610543576040516314815f4760e31b815260040160405180910390fd5b805160208201fd5b92915050565b6000600181610568610374368a90038a018a610a65565b8152602001908152602001600020600081548092919061058790610b05565b9091555063b47b2fb160e01b98975050505050505050565b6000816040516020016105f8919081516001600160a01b03908116825260208084015182169083015260408084015162ffffff169083015260608084015160020b90830152608092830151169181019190915260a00190565b604051602081830303815290604052805190602001209050919050565b6001600160a01b038116811461062a57600080fd5b50565b803561063881610615565b919050565b600060a0828403121561064f57600080fd5b50919050565b60006060828403121561064f57600080fd5b60008083601f84011261067957600080fd5b50813567ffffffffffffffff81111561069157600080fd5b6020830191508360208285010111156106a957600080fd5b9250929050565b600080600080600061014086880312156106c957600080fd5b85356106d481610615565b94506106e3876020880161063d565b93506106f28760c08801610655565b925061012086013567ffffffffffffffff81111561070f57600080fd5b61071b88828901610667565b969995985093965092949392505050565b6000806000806000610100868803121561074557600080fd5b853561075081610615565b945061075f876020880161063d565b935060c086013561076f81610615565b925060e086013567ffffffffffffffff81111561070f57600080fd5b60006020828403121561079d57600080fd5b5035919050565b8035600281900b811461063857600080fd5b60008060008060008061012087890312156107d057600080fd5b86356107db81610615565b95506107ea886020890161063d565b945060c08701356107fa81610615565b935061080860e088016107a4565b925061010087013567ffffffffffffffff81111561082557600080fd5b61083189828a01610667565b979a9699509497509295939492505050565b6000806020838503121561085657600080fd5b823567ffffffffffffffff81111561086d57600080fd5b61087985828601610667565b90969095509350505050565b600060208083528351808285015260005b818110156108b257858101830151858201604001528201610896565b506000604082860101526040601f19601f8301168501019250505092915050565b60008060008060008061016087890312156108ed57600080fd5b86356108f881610615565b9550610907886020890161063d565b94506109168860c08901610655565b9350610120870135925061014087013567ffffffffffffffff81111561082557600080fd5b600080600080600080610120878903121561095557600080fd5b863561096081610615565b955061096f886020890161063d565b945060c0870135935060e0870135925061010087013567ffffffffffffffff81111561082557600080fd5b815115158152610140810160208301516109b8602084018215159052565b5060408301516109cc604084018215159052565b5060608301516109e0606084018215159052565b5060808301516109f4608084018215159052565b5060a0830151610a0860a084018215159052565b5060c0830151610a1c60c084018215159052565b5060e0830151610a3060e084018215159052565b5061010083810151151590830152610120928301511515929091019190915290565b803562ffffff8116811461063857600080fd5b600060a08284031215610a7757600080fd5b60405160a0810181811067ffffffffffffffff82111715610aa857634e487b7160e01b600052604160045260246000fd5b6040528235610ab681610615565b81526020830135610ac681610615565b6020820152610ad760408401610a52565b6040820152610ae8606084016107a4565b6060820152610af96080840161062d565b60808201529392505050565b600060018201610b2557634e487b7160e01b600052601160045260246000fd5b5060010190565b818382376000910190815291905056fea2646970667358221220df0fbdd29be3dedf804ada37c82e9656ba66c79d191d4eda34cd2497a6822f3b64736f6c63430008130033a26469706673582212207c16179daa855bc5820d734a67986cc6d8b5a42d7134a14d300e45f53513d81064736f6c63430008130033";

type CounterScriptConstructorParams =
  | [signer?: Signer]
  | ConstructorParameters<typeof ContractFactory>;

const isSuperArgs = (
  xs: CounterScriptConstructorParams
): xs is ConstructorParameters<typeof ContractFactory> => xs.length > 1;

export class CounterScript__factory extends ContractFactory {
  constructor(...args: CounterScriptConstructorParams) {
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
      CounterScript & {
        deploymentTransaction(): ContractTransactionResponse;
      }
    >;
  }
  override connect(runner: ContractRunner | null): CounterScript__factory {
    return super.connect(runner) as CounterScript__factory;
  }

  static readonly bytecode = _bytecode;
  static readonly abi = _abi;
  static createInterface(): CounterScriptInterface {
    return new Interface(_abi) as CounterScriptInterface;
  }
  static connect(
    address: string,
    runner?: ContractRunner | null
  ): CounterScript {
    return new Contract(address, _abi, runner) as unknown as CounterScript;
  }
}
