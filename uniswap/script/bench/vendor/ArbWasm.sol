// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.17;

/// @notice The parts of Arbitrum's ArbWasm precompile that `StylusDeployer` needs.
/// @dev Trimmed from https://github.com/OffchainLabs/nitro-contracts `src/precompiles/ArbWasm.sol`.
interface ArbWasm {
    /// @notice Compile and activate a Stylus contract, paying the data fee out of the call's value.
    function activateProgram(address program)
        external
        payable
        returns (uint16 version, uint256 dataFee);

    /// @notice The Stylus version a given codehash was activated with, if any.
    function codehashVersion(bytes32 codehash) external view returns (uint16 version);

    /// @notice The Stylus version this chain is on.
    function stylusVersion() external view returns (uint16 version);
}
