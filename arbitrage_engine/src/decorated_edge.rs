use blake3::{Hash, Hasher};

/// Edge with metadata
/// petgraph::add_edge returns EdgeIndex<Ix> type
/// We will maintain an in-memory hashmap of EdgeIndex<Ix> => DecoratedEdge
/// Create class to store these data structures
#[derive(Debug)]
pub struct DecoratedEdge<'a> {
    weight: f64,
    /// int enum for protocol
    protocol_type: usize,
    /// int enum for blockchain node type (e.g. EVM, Solana)
    node_type: usize,
    /// Hex string address for pool
    pool_address: &'a str,
    /// Miscellaneous data
    data: &'a str,
}

impl DecoratedEdge<'_> {
    fn get_unique_id(protocol_type: usize, node_type: usize, pool_address: &str) -> Hash {
        let mut hasher = Hasher::new();
        hasher.update(&protocol_type.to_ne_bytes());
        hasher.update(&node_type.to_ne_bytes());
        hasher.update(pool_address.as_bytes());
        return hasher.finalize();
    }
}
