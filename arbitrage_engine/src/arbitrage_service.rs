// - edgeIndex -> decoratedEdge ()
// - (nodeId, nodeId, blockchainType, protocolType) -> edgeIndex (Does edge exist?)

use petgraph::{
    graph::Graph,
    prelude::{EdgeIndex, NodeIndex},
};
use blake3::Hash;
use std::collections::HashMap;
use super::decorated_edge::DecoratedEdge;

/// Point of contact interacting with the arbitrage functionality
struct ArbitrageService<'a> {
    /// Core graph data structure on which we perform the algorithm
    /// &str for nodeId, f64 for edge weight
    graph: Graph<&'a str, f64>,
    /// nodeId => NodeIndex
    /// 'Have we added this nodeId previously?`
    node_indexes: HashMap<&'a str, NodeIndex>,
    /// When we have found arbitrage path, resolve NodeIndex => nodeId
    nodes: HashMap<NodeIndex, &'a str>,
    /// DecoratedEdgeHash => EdgeIndex, 'Have we added this edge previously'?
    edge_hashes: HashMap<Hash, EdgeIndex>,
    /// EdgeIndex => DecoratedEdge
    decorated_edges: HashMap<EdgeIndex, DecoratedEdge<'a>>,
}

impl<'a> ArbitrageService <'a> {
    fn new() -> Self {
        ArbitrageService {
            graph: Graph::new(),
            node_indexes: HashMap::new(),
            nodes: HashMap::new(),
            edge_hashes: HashMap::new(),
            decorated_edges: HashMap::new(),
        }
    }
}

