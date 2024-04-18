pragma solidity ^0.8.19;

interface IDeploy {
    function deploy(
        bytes calldata _data,
        uint256 _salt
    ) external payable returns (address);
}

contract Redeploy {
    IDeploy public IDEPLOY =
        IDeploy(0x4e59b44847b379578588920cA78FbF26c0B4956C);

    function deploy(
        address from,
        uint256 salt
    ) public payable returns (address) {
        bytes memory code = from.code;
        return IDEPLOY.deploy(code, salt);
    }
}
