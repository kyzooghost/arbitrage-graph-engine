Proof-of-concept arbitrage scanner for blockchain decentralised exchanges (DEXs), built in Rust.

WIP.

Modules
- Core arbitrage engine
- Adaptors for each network and protocol

Useful CLI commands


Run unit tests without silencing stdout

```
cargo test -- --nocapture
```

cargo test uniswap_v2_basic_integration_test -- --nocapture --quiet