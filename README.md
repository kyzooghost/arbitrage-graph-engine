Proof-of-concept arbitrage scanner for blockchain decentralised exchanges (DEXs), built in Rust.

WIP.

Modules
- Core arbitrage engine
- (Deprecated) Adaptors for each network and protocol

# Goal

Hold graph data structure in memory. Receive messages from another program to
i.) Update graph data structure
ii.) Run arbitrage-scan algorithm on in-memory data structure, return discovered arbitrages

Can model as request-response endpoints

API:
- add_node<T, T>
- add_edge<T, f64>(T, T, f64)
- get_negative_cycle_quick

# TODO

Loop in main.rs that waits on messages from another process

Handler for external process messages
-> API for arbitrage_engine