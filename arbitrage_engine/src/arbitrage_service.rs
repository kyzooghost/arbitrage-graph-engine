use crate::{
    decorated_edge::DecoratedEdge,
    engine::{get_all_negative_cycles_0, get_negative_cycle_quick},
    path::{DecoratedPath, Path}
};
use blake3::Hash;
use petgraph::{
    graph::{EdgeReference, Graph},
    prelude::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
};
use std::collections::HashMap;

pub trait IArbitrageService {
    fn upsert_path(&mut self, n0: &str, n1: &str, edge: DecoratedEdge) -> bool;
    /// Returns all arbitrages found
    fn scan_arbitrages(&self) -> Vec<DecoratedPath>;
    /// Stops at first arbitrage found
    fn scan_arbitrages_quick(&self) -> Vec<DecoratedPath>;
    fn _decorate_paths(&self, path: Vec<Path<String>>) -> Vec<DecoratedPath>;
    // TODO - Provide streaming API for scan_arbitrages, so that we can use results as they become available without having to wait for entire algorithm to run
}

/// Point of contact interacting with the arbitrage functionality
pub struct ArbitrageService {
    /// Core directed graph data structure on which we perform the algorithm
    /// &str for nodeId, f64 for edge weight
    graph: Graph<String, f64>,
    /// nodeId => NodeIndex
    /// 'Have we added this nodeId previously?`
    node_indexes: HashMap<String, NodeIndex>,
    /// When we have found arbitrage path, resolve NodeIndex => nodeId
    nodes: HashMap<NodeIndex, String>,
    /// DecoratedEdgeHash => EdgeIndex, 'Have we added this edge previously'?
    edge_indexes: HashMap<Hash, EdgeIndex>,
    /// EdgeIndex => DecoratedEdge
    decorated_edges: HashMap<EdgeIndex, DecoratedEdge>,
}

impl ArbitrageService {
    pub fn new() -> Self {
        ArbitrageService {
            graph: Graph::new(),
            node_indexes: HashMap::new(),
            nodes: HashMap::new(),
            edge_indexes: HashMap::new(),
            decorated_edges: HashMap::new(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.node_count()
    }
}

impl Default for ArbitrageService {
    fn default() -> Self {
        ArbitrageService::new()
    }
}

impl IArbitrageService for ArbitrageService {
    /// Add edge from n0 -> n1
    /// We will not add the reverse edge here - up to the client
    /// Return true if graph updated, false if not
    fn upsert_path(&mut self, n0: &str, n1: &str, edge: DecoratedEdge) -> bool {
        // If current edge exists, nothing to do
        let edge_hash = DecoratedEdge::get_unique_id(&edge);
        if self.edge_indexes.contains_key(&edge_hash) {
            return false;
        };

        // Add n0 if not yet existing
        let n0_index: NodeIndex = if let Some(index) = self.node_indexes.get(n0) {
            *index
        } else {
            // Add to graph
            let new_index = self.graph.add_node(n0.to_string());
            // Add to node_indexes
            self.node_indexes.insert(n0.to_string(), new_index);
            // Add to nodes
            self.nodes.insert(new_index, n0.to_string());
            *self.node_indexes.get(n0).unwrap()
        };

        // Add n1 if not yet existing
        let n1_index: NodeIndex = if let Some(index) = self.node_indexes.get(n1) {
            *index
        } else {
            // Add to graph
            let new_index = self.graph.add_node(n1.to_string());
            // Add to node_indexes
            self.node_indexes.insert(n1.to_string(), new_index);
            // Add to nodes
            self.nodes.insert(new_index, n1.to_string());
            *self.node_indexes.get(n1).unwrap()
        };

        // Iterate over existing edges n0 -> n1, we only want 'two most extreme edge weights' to be present
        let existing_edges_iterator_0 = self.graph.edges_connecting(n0_index, n1_index);
        let existing_edges_iterator_0_count = existing_edges_iterator_0.clone().count();

        // 0 or 1 existing edges -> straightforward add candidate edge
        if existing_edges_iterator_0_count < 2 {
            let new_edge_index = self.graph.add_edge(n0_index, n1_index, edge.weight);
            self.edge_indexes.insert(edge_hash, new_edge_index);
            self.decorated_edges.insert(new_edge_index, edge);
            return true;
        }

        // 2 existing edges + 1 new candidate edge -> delete existing edge A, delete existing edge B, or do nothing
        let mut existing_edges: Vec<EdgeReference<'_, f64>> = existing_edges_iterator_0.collect();
        assert!(
            existing_edges.len() == 2,
            "arbitrage_service.upsert_path() - More than 2 existing edges between {} and {}",
            n0,
            n1
        );
        // TODO - Check that this actually sorts as expected
        existing_edges.sort_by(|a, b| a.weight().total_cmp(b.weight()));

        // Replace existing_edges[0]
        // Don't use `remove_edge()` because it invalidates the last edge index in the graph
        if edge.weight < *existing_edges[0].weight() {
            let existing_edge_index = existing_edges[0].id();

            // Update graph edge weight
            let edge_weight_to_update = self.graph.edge_weight_mut(existing_edge_index).unwrap();
            *edge_weight_to_update = edge.weight;
            let existing_decorated_edge = self.decorated_edges.get(&existing_edge_index).unwrap();
            // Update edge_indexes
            let existing_edge_hash = DecoratedEdge::get_unique_id(existing_decorated_edge);
            self.edge_indexes.remove(&existing_edge_hash);
            self.edge_indexes.insert(edge_hash, existing_edge_index);
            // Update decorated_edges
            self.decorated_edges.insert(existing_edge_index, edge);

            return true;
        // Replace existing_edges[1]
        } else if edge.weight > *existing_edges[1].weight() {
            let existing_edge_index = existing_edges[1].id();

            // Update graph edge weight
            let edge_weight_to_update = self.graph.edge_weight_mut(existing_edge_index).unwrap();
            *edge_weight_to_update = edge.weight;
            let existing_decorated_edge = self.decorated_edges.get(&existing_edge_index).unwrap();
            // Update edge_indexes
            let existing_edge_hash = DecoratedEdge::get_unique_id(existing_decorated_edge);
            self.edge_indexes.remove(&existing_edge_hash);
            self.edge_indexes.insert(edge_hash, existing_edge_index);
            // Update decorated_edges
            self.decorated_edges.insert(existing_edge_index, edge);

            return true;
        // Do nothing
        } else {
            return false;
        }
    }

    fn scan_arbitrages_quick(&self) -> Vec<DecoratedPath> {
        let (_, path_option) = get_negative_cycle_quick(&self.graph);
        match path_option {
            None => Vec::new(),
            Some(path) => Self::_decorate_paths(self, vec![path]),
        }
    }

    fn scan_arbitrages(&self) -> Vec<DecoratedPath> {
        let path = get_all_negative_cycles_0(&self.graph);
        return Self::_decorate_paths(self, path);
    }

    fn _decorate_paths(&self, path_collection: Vec<Path<String>>) -> Vec<DecoratedPath> {
        let edge_index_to_decorated_edge =
            |index: EdgeIndex| self.decorated_edges.get(&index).unwrap();

        let node_index_to_node = |index: NodeIndex| self.nodes.get(&index).unwrap();

        path_collection
            .into_iter()
            .map(|path| DecoratedPath {
                edges: path
                    .edges()
                    .into_iter()
                    .map(edge_index_to_decorated_edge)
                    .collect(),
                nodes: path.nodes().into_iter().map(node_index_to_node).collect(),
            })
            .collect::<Vec<DecoratedPath>>()
    }
}
