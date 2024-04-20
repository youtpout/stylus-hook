PRIVATE_KEY=0x0123456789012345678901234567890123456789012345678901234567890123
cargo stylus check

deployCreate2() {
    echo "Deploying create"
    cargo stylus deploy --private-key $PRIVATE_KEY --wasm-file-path target/wasm32-unknown-unknown/release/stylus_create2.wasm -e http://localhost:8547/
    echo "Proxy deployed"
}