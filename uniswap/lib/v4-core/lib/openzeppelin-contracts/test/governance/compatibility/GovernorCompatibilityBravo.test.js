const { BN, expectEvent, expectRevert } = require('@openzeppelin/test-helpers');
const Enums = require('../../helpers/enums');
const RLP = require('rlp');

const {
  runGovernorWorkflow,
} = require('../GovernorWorkflow.behavior');

const Token = artifacts.require('ERC20VotesCompMock');
const Timelock = artifacts.require('CompTimelock');
const Governor = artifacts.require('GovernorCompatibilityBravoMock');
const CallReceiver = artifacts.require('CallReceiverMock');

function makeContractAddress (creator, nonce) {
  return web3.utils.toChecksumAddress(web3.utils.sha3(RLP.encode([creator, nonce])).slice(12).substring(14));
}

contract('GovernorCompatibilityBravo', function (accounts) {
  const [ owner, proposer, voter1, voter2, voter3, voter4, other ] = accounts;

  const name = 'OZ-Governor';
  // const version = '1';
  const tokenName = 'MockToken';
  const tokenSymbol = 'MTKN';
  const tokenSupply = web3.utils.toWei('100');
  const proposalThreshold = web3.utils.toWei('10');

  beforeEach(async function () {
    const [ deployer ] = await web3.eth.getAccounts();

    this.token = await Token.new(tokenName, tokenSymbol);

    // Need to predict governance address to set it as timelock admin with a delayed transfer
    const nonce = await web3.eth.getTransactionCount(deployer);
    const predictGovernor = makeContractAddress(deployer, nonce + 1);

    this.timelock = await Timelock.new(predictGovernor, 2 * 86400);
    this.mock = await Governor.new(name, this.token.address, 4, 16, proposalThreshold, this.timelock.address);
    this.receiver = await CallReceiver.new();
    await this.token.mint(owner, tokenSupply);
    await this.token.delegate(voter1, { from: voter1 });
    await this.token.delegate(voter2, { from: voter2 });
    await this.token.delegate(voter3, { from: voter3 });
    await this.token.delegate(voter4, { from: voter4 });

    await this.token.transfer(proposer, proposalThreshold, { from: owner });
    await this.token.delegate(proposer, { from: proposer });
  });

  it('deployment check', async function () {
    expect(await this.mock.name()).to.be.equal(name);
    expect(await this.mock.token()).to.be.equal(this.token.address);
    expect(await this.mock.votingDelay()).to.be.bignumber.equal('4');
    expect(await this.mock.votingPeriod()).to.be.bignumber.equal('16');
    expect(await this.mock.quorum(0)).to.be.bignumber.equal('0');
    expect(await this.mock.quorumVotes()).to.be.bignumber.equal('0');
    expect(await this.mock.COUNTING_MODE()).to.be.equal('support=bravo&quorum=bravo');
  });

  describe('nominal', function () {
    beforeEach(async function () {
      this.settings = {
        proposal: [
          [ this.receiver.address ], // targets
          [ web3.utils.toWei('0') ], // values
          [ this.receiver.contract.methods.mockFunction().encodeABI() ], // calldatas
          '<proposal description>', // description
        ],
        proposer,
        tokenHolder: owner,
        voters: [
          {
            voter: voter1,
            weight: web3.utils.toWei('1'),
            support: Enums.VoteType.Abstain,
          },
          {
            voter: voter2,
            weight: web3.utils.toWei('10'),
            support: Enums.VoteType.For,
          },
          {
            voter: voter3,
            weight: web3.utils.toWei('5'),
            support: Enums.VoteType.Against,
          },
          {
            voter: voter4,
            support: '100',
            error: 'GovernorCompatibilityBravo: invalid vote type',
          },
          {
            voter: voter1,
            support: Enums.VoteType.For,
            error: 'GovernorCompatibilityBravo: vote already cast',
            skip: true,
          },
        ],
        steps: {
          queue: { delay: 7 * 86400 },
        },
      };
      this.votingDelay = await this.mock.votingDelay();
      this.votingPeriod = await this.mock.votingPeriod();
      this.receipts = {};
    });
    afterEach(async function () {
      const proposal = await this.mock.proposals(this.id);
      expect(proposal.id).to.be.bignumber.equal(this.id);
      expect(proposal.proposer).to.be.equal(proposer);
      expect(proposal.eta).to.be.bignumber.equal(this.eta);
      expect(proposal.startBlock).to.be.bignumber.equal(this.snapshot);
      expect(proposal.endBlock).to.be.bignumber.equal(this.deadline);
      expect(proposal.canceled).to.be.equal(false);
      expect(proposal.executed).to.be.equal(true);

      for (const [key, value] of Object.entries(Enums.VoteType)) {
        expect(proposal[`${key.toLowerCase()}Votes`]).to.be.bignumber.equal(
          Object.values(this.settings.voters).filter(({ support }) => support === value).reduce(
            (acc, { weight }) => acc.add(new BN(weight)),
            new BN('0'),
          ),
        );
      }

      const action = await this.mock.getActions(this.id);
      expect(action.targets).to.be.deep.equal(this.settings.proposal[0]);
      // expect(action.values).to.be.deep.equal(this.settings.proposal[1]);
      expect(action.signatures).to.be.deep.equal(Array(this.settings.proposal[2].length).fill(''));
      expect(action.calldatas).to.be.deep.equal(this.settings.proposal[2]);

      for (const voter of this.settings.voters.filter(({ skip }) => !skip)) {
        expect(await this.mock.hasVoted(this.id, voter.voter)).to.be.equal(voter.error === undefined);

        const receipt = await this.mock.getReceipt(this.id, voter.voter);
        expect(receipt.hasVoted).to.be.equal(voter.error === undefined);
        expect(receipt.support).to.be.bignumber.equal(voter.error === undefined ? voter.support : '0');
        expect(receipt.votes).to.be.bignumber.equal(voter.error === undefined ? voter.weight : '0');
      }

      expectEvent(
        this.receipts.propose,
        'ProposalCreated',
        {
          proposalId: this.id,
          proposer,
          targets: this.settings.proposal[0],
          // values: this.settings.proposal[1].map(value => new BN(value)),
          signatures: this.settings.proposal[2].map(() => ''),
          calldatas: this.settings.proposal[2],
          startBlock: new BN(this.receipts.propose.blockNumber).add(this.votingDelay),
          endBlock: new BN(this.receipts.propose.blockNumber).add(this.votingDelay).add(this.votingPeriod),
          description: this.settings.proposal[3],
        },
      );

      this.receipts.castVote.filter(Boolean).forEach(vote => {
        const { voter } = vote.logs.find(Boolean).args;
        expectEvent(
          vote,
          'VoteCast',
          this.settings.voters.find(({ address }) => address === voter),
        );
      });
      expectEvent(
        this.receipts.execute,
        'ProposalExecuted',
        { proposalId: this.id },
      );
      await expectEvent.inTransaction(
        this.receipts.execute.transactionHash,
        this.receiver,
        'MockFunctionCalled',
      );
    });
    runGovernorWorkflow();
  });

  describe('with function selector and arguments', function () {
    beforeEach(async function () {
      this.settings = {
        proposal: [
          Array(4).fill(this.receiver.address),
          Array(4).fill(web3.utils.toWei('0')),
          [
            '',
            '',
            'mockFunctionNonPayable()',
            'mockFunctionWithArgs(uint256,uint256)',
          ],
          [
            this.receiver.contract.methods.mockFunction().encodeABI(),
            this.receiver.contract.methods.mockFunctionWithArgs(17, 42).encodeABI(),
            '0x',
            web3.eth.abi.encodeParameters(['uint256', 'uint256'], [18, 43]),
          ],
          '<proposal description>', // description
        ],
        proposer,
        tokenHolder: owner,
        voters: [
          {
            voter: voter1,
            weight: web3.utils.toWei('10'),
            support: Enums.VoteType.For,
          },
        ],
        steps: {
          queue: { delay: 7 * 86400 },
        },
      };
    });
    runGovernorWorkflow();
    afterEach(async function () {
      await expectEvent.inTransaction(
        this.receipts.execute.transactionHash,
        this.receiver,
        'MockFunctionCalled',
      );
      await expectEvent.inTransaction(
        this.receipts.execute.transactionHash,
        this.receiver,
        'MockFunctionCalled',
      );
      await expectEvent.inTransaction(
        this.receipts.execute.transactionHash,
        this.receiver,
        'MockFunctionCalledWithArgs',
        { a: '17', b: '42' },
      );
      await expectEvent.inTransaction(
        this.receipts.execute.transactionHash,
        this.receiver,
        'MockFunctionCalledWithArgs',
        { a: '18', b: '43' },
      );
    });
  });

  describe('proposalThreshold not reached', function () {
    beforeEach(async function () {
      this.settings = {
        proposal: [
          [ this.receiver.address ], // targets
          [ web3.utils.toWei('0') ], // values
          [ this.receiver.contract.methods.mockFunction().encodeABI() ], // calldatas
          '<proposal description>', // description
        ],
        proposer: other,
        steps: {
          propose: { error: 'GovernorCompatibilityBravo: proposer votes below proposal threshold' },
          wait: { enable: false },
          queue: { enable: false },
          execute: { enable: false },
        },
      };
    });
    runGovernorWorkflow();
  });

  describe('cancel', function () {
    beforeEach(async function () {
      this.settings = {
        proposal: [
          [ this.receiver.address ], // targets
          [ web3.utils.toWei('0') ], // values
          [ this.receiver.contract.methods.mockFunction().encodeABI() ], // calldatas
          '<proposal description>', // description
        ],
        proposer,
        tokenHolder: owner,
        steps: {
          wait: { enable: false },
          queue: { enable: false },
          execute: { enable: false },
        },
      };
    });

    describe('by proposer', function () {
      afterEach(async function () {
        await this.mock.cancel(this.id, { from: proposer });
      });
      runGovernorWorkflow();
    });

    describe('if proposer below threshold', function () {
      afterEach(async function () {
        await this.token.transfer(voter1, web3.utils.toWei('1'), { from: proposer });
        await this.mock.cancel(this.id);
      });
      runGovernorWorkflow();
    });

    describe('not if proposer above threshold', function () {
      afterEach(async function () {
        await expectRevert(
          this.mock.cancel(this.id),
          'GovernorBravo: proposer above threshold',
        );
      });
      runGovernorWorkflow();
    });
  });

  describe('with compatibility interface', function () {
    beforeEach(async function () {
      this.settings = {
        proposal: [
          [ this.receiver.address ], // targets
          [ web3.utils.toWei('0') ], // values
          [ 'mockFunction()' ], // signatures
          [ '0x' ], // calldatas
          '<proposal description>', // description
        ],
        proposer,
        tokenHolder: owner,
        voters: [
          {
            voter: voter1,
            weight: web3.utils.toWei('1'),
            support: Enums.VoteType.Abstain,
          },
          {
            voter: voter2,
            weight: web3.utils.toWei('10'),
            support: Enums.VoteType.For,
          },
          {
            voter: voter3,
            weight: web3.utils.toWei('5'),
            support: Enums.VoteType.Against,
          },
          {
            voter: voter4,
            support: '100',
            error: 'GovernorCompatibilityBravo: invalid vote type',
          },
          {
            voter: voter1,
            support: Enums.VoteType.For,
            error: 'GovernorCompatibilityBravo: vote already cast',
            skip: true,
          },
        ],
        steps: {
          queue: { delay: 7 * 86400 },
        },
      };
      this.votingDelay = await this.mock.votingDelay();
      this.votingPeriod = await this.mock.votingPeriod();
      this.receipts = {};
    });

    afterEach(async function () {
      const proposal = await this.mock.proposals(this.id);
      expect(proposal.id).to.be.bignumber.equal(this.id);
      expect(proposal.proposer).to.be.equal(proposer);
      expect(proposal.eta).to.be.bignumber.equal(this.eta);
      expect(proposal.startBlock).to.be.bignumber.equal(this.snapshot);
      expect(proposal.endBlock).to.be.bignumber.equal(this.deadline);
      expect(proposal.canceled).to.be.equal(false);
      expect(proposal.executed).to.be.equal(true);

      for (const [key, value] of Object.entries(Enums.VoteType)) {
        expect(proposal[`${key.toLowerCase()}Votes`]).to.be.bignumber.equal(
          Object.values(this.settings.voters).filter(({ support }) => support === value).reduce(
            (acc, { weight }) => acc.add(new BN(weight)),
            new BN('0'),
          ),
        );
      }

      const action = await this.mock.getActions(this.id);
      expect(action.targets).to.be.deep.equal(this.settings.proposal[0]);
      // expect(action.values).to.be.deep.equal(this.settings.proposal[1]);
      expect(action.signatures).to.be.deep.equal(this.settings.proposal[2]);
      expect(action.calldatas).to.be.deep.equal(this.settings.proposal[3]);

      for (const voter of this.settings.voters.filter(({ skip }) => !skip)) {
        expect(await this.mock.hasVoted(this.id, voter.voter)).to.be.equal(voter.error === undefined);

        const receipt = await this.mock.getReceipt(this.id, voter.voter);
        expect(receipt.hasVoted).to.be.equal(voter.error === undefined);
        expect(receipt.support).to.be.bignumber.equal(voter.error === undefined ? voter.support : '0');
        expect(receipt.votes).to.be.bignumber.equal(voter.error === undefined ? voter.weight : '0');
      }

      expectEvent(
        this.receipts.propose,
        'ProposalCreated',
        {
          proposalId: this.id,
          proposer,
          targets: this.settings.proposal[0],
          // values: this.settings.proposal[1].map(value => new BN(value)),
          signatures: this.settings.proposal[2].map(_ => ''),
          calldatas: this.settings.shortProposal[2],
          startBlock: new BN(this.receipts.propose.blockNumber).add(this.votingDelay),
          endBlock: new BN(this.receipts.propose.blockNumber).add(this.votingDelay).add(this.votingPeriod),
          description: this.settings.proposal[4],
        },
      );

      this.receipts.castVote.filter(Boolean).forEach(vote => {
        const { voter } = vote.logs.find(Boolean).args;
        expectEvent(
          vote,
          'VoteCast',
          this.settings.voters.find(({ address }) => address === voter),
        );
      });
      expectEvent(
        this.receipts.execute,
        'ProposalExecuted',
        { proposalId: this.id },
      );
      await expectEvent.inTransaction(
        this.receipts.execute.transactionHash,
        this.receiver,
        'MockFunctionCalled',
      );
    });
    runGovernorWorkflow();
  });
});
