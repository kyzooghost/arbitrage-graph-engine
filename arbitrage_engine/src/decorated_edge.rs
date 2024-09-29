use blake3::{Hash, Hasher};

/// Edge with metadata
/// petgraph::add_edge returns EdgeIndex<Ix> type
/// We will maintain an in-memory hashmap of EdgeIndex<Ix> => DecoratedEdge
/// Create class to store these data structures
#[derive(Debug)]
pub struct DecoratedEdge {
    pub weight: f64,
    /// int enum for protocol
    protocol_type: usize,
    /// int enum for blockchain node type (e.g. EVM, Solana)
    node_type: usize,
    /// Hex string address for pool
    pool_address: String,
    /// Miscellaneous data
    data: String,
}

impl DecoratedEdge {
    pub fn get_unique_id(edge: &DecoratedEdge) -> Hash {
        let mut hasher = Hasher::new();
        hasher.update(&edge.protocol_type.to_ne_bytes());
        hasher.update(&edge.node_type.to_ne_bytes());
        hasher.update(edge.pool_address.as_bytes());

        hasher.finalize()
    }
}
