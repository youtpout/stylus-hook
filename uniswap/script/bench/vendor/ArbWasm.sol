// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

/// @notice The three methods of Arbitrum's ArbWasm precompile that `StylusDeployer` calls.
/// @dev Declared here from the precompile's published ABI rather than copied: nitro-contracts is
///      BUSL-1.1, and this repository is MIT. An earlier revision of this file carried the BUSL
///      tag by mistake, which would have put a production-use restriction on an MIT repository.
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
