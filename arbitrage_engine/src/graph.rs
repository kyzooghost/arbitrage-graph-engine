pub use graph_mod::*;

mod graph_mod {
    use petgraph::{
        graph::Graph,
        prelude::{NodeIndex, EdgeIndex},
        Direction::Outgoing,
        visit::EdgeRef
    };

    use super::super::utils::logger::{logObject, logText};

    #[test]
    fn petgraph_basic_methods() {
        let mut graph: Graph<&str, f64> = Graph::new();
        let origin = graph.add_node("Denver");
        let destination_1 = graph.add_node("San Diego");
        let destination_2 = graph.add_node("New York");
        let cost_1 = graph.add_edge(origin, destination_1, 250.0);
        let cost_2 = graph.add_edge(origin, destination_2, 1099.0);
        assert_eq!(graph.node_weight(origin).unwrap(), &"Denver");
        assert_eq!(graph[destination_1], "San Diego");
        assert_eq!(graph.edge_weight(cost_1).unwrap(), &250.0);
        assert_eq!(graph.edge_weight(cost_2).unwrap(), &1099.0);
        *graph.edge_weight_mut(cost_1).unwrap() = 249.0;
        assert_eq!(graph.edge_weight(cost_1).unwrap(), &249.0);
    }

    pub mod path {
        use super::{Graph, NodeIndex, EdgeIndex};
        use std::marker::PhantomData;

        // Choosing to store vector of indexes, rather than using a recursive type as we did in Java. Should be ok if the vectors don't grow too large.
        #[derive(Debug)]
        pub struct Path<N> {
            weight: f64,
            edges: Vec<EdgeIndex>,
            nodes: Vec<NodeIndex>,
            node_type: PhantomData<N>
        }

        impl<N> Path<N> {
            pub fn new(source_node: NodeIndex) -> Self {
                Path {
                    weight: 0.0_f64,
                    edges: Vec::new(),
                    nodes: vec![source_node],
                    node_type: PhantomData
                }
            }

            pub fn add_to_path(&mut self, graph: &Graph<N, f64>, edge: EdgeIndex) {
                assert_eq!(&graph.edge_endpoints(edge).unwrap().0, self.nodes.last().unwrap(), "Edge does not extend from existing path");
                self.weight += graph.edge_weight(edge).unwrap();
                self.edges.push(edge);
                self.nodes.push(graph.edge_endpoints(edge).unwrap().1);
            }

            pub fn weight(&self) -> f64 {
                self.weight
            }

            pub fn edges(&self) -> Vec<EdgeIndex> {
                let mut vec:Vec<EdgeIndex> = Vec::new();
                for edge in self.edges.iter() {
                    vec.push(*edge);
                }
                vec
            }

            pub fn nodes(&self) -> Vec<NodeIndex> {
                let mut vec:Vec<NodeIndex> = Vec::new();
                for node in self.nodes.iter() {
                    vec.push(*node);
                }
                vec
            }

            pub fn length(&self) -> usize {
                self.nodes.len()
            }
        }

        #[test]
        fn path_basic_methods() {
            // Build graph
            let mut graph: Graph<&str, f64> = Graph::new();
            let origin = graph.add_node("Denver");
            let destination_1 = graph.add_node("San Diego");
            let destination_2 = graph.add_node("New York");
            let cost_1 = graph.add_edge(origin, destination_1, 250.0);
            let cost_2 = graph.add_edge(origin, destination_2, 1099.0);

            // Create path
            let mut path: Path<&str> = Path::new(origin);
            assert_eq!(path.weight(), 0.0);
            assert_eq!(path.length(), 1);
            assert_eq!(path.edges().len(), 0);
            assert_eq!(path.nodes().len(), 1);
            assert_eq!(path.nodes()[0], origin);

            // Add to path
            path.add_to_path(&graph, cost_1);
            assert_eq!(path.weight(), 250.0);
            assert_eq!(path.length(), 2);
            assert_eq!(path.edges().len(), 1);
            assert_eq!(path.nodes().len(), 2);
            assert_eq!(path.nodes()[0], origin);
            assert_eq!(path.nodes()[1], destination_1);
            assert_eq!(path.edges()[0], cost_1);

            // Create another path
            let mut path1: Path<&str> = Path::new(origin);
            assert_eq!(path1.weight(), 0.0);
            assert_eq!(path1.length(), 1);
            assert_eq!(path1.edges().len(), 0);
            assert_eq!(path1.nodes().len(), 1);
            assert_eq!(path1.nodes()[0], origin);

            // Add to another path
            path1.add_to_path(&graph, cost_2);
            assert_eq!(path1.weight(), 1099.0);
            assert_eq!(path1.length(), 2);
            assert_eq!(path1.edges().len(), 1);
            assert_eq!(path1.nodes().len(), 2);
            assert_eq!(path1.nodes()[0], origin);
            assert_eq!(path1.nodes()[1], destination_2);
            assert_eq!(path1.edges()[0], cost_2);
        }
    }

    pub mod cycle {
        use super::{Graph, path::Path, NodeIndex, EdgeIndex, Outgoing, EdgeRef, logObject, logText};
        use std::{collections::HashMap};

        // DFS algorithm to determine if a cycle exists in a graph, linear time algorithm: O(V + E).
        // Returns tuple
        // tuple.0 (bool): false if no cycle found, true if cycle present.
        // tuple.1 (Option<Path<N>>): None if no cycle found, Path representing cycle if cycle found.
        pub fn has_cycle<N>(graph: &Graph<N, f64>) -> (bool, Option<Path<N>>) {
            // Initialise data structures
            let mut visited: HashMap<NodeIndex, bool> = HashMap::new();
            let mut edgeTo: HashMap<NodeIndex, Option<EdgeIndex>> = HashMap::new();
            let mut onStack: HashMap<NodeIndex, bool> = HashMap::new();
            let mut cycle: Option<Path<N>> = None;

            for node in graph.node_indices() {
                visited.insert(node, false);
                edgeTo.insert(node, None);
                onStack.insert(node, false);
            }

            for node in graph.node_indices() {
                if !visited.get(&node).unwrap() {
                    _has_cycle_dfs(graph, node, &mut visited, &mut edgeTo, &mut onStack, &mut cycle);
                }
            }

            match cycle {
                None => {(false, None)},
                Some(discovered_cycle) => {(true, Some(discovered_cycle))}
            }
        }

        fn _has_cycle_dfs<N>(
            graph: &Graph<N, f64>, 
            node: NodeIndex, 
            visited: &mut HashMap<NodeIndex, bool>, 
            edgeTo: &mut HashMap<NodeIndex, Option<EdgeIndex>>, 
            onStack: &mut HashMap<NodeIndex, bool>,
            cycle: &mut Option<Path<N>>
        ) {
            // logObject("dfs visit node: ", &node);
            onStack.insert(node, true);
            visited.insert(node, true);

            for edge in graph.edges_directed(node, Outgoing) {
                let edgeId = edge.id();
                let target = edge.target();

                if cycle.is_some() {
                    return;
                } else if !visited.get(&target).unwrap() {
                    edgeTo.insert(target, Some(edgeId));
                    _has_cycle_dfs(graph, target, visited, edgeTo, onStack, cycle);
                } else if *onStack.get(&target).unwrap() {
                    let mut new_cycle: Path<N> = Path::new(target);
                    let mut edgeStack: Vec<EdgeIndex>= Vec::new();

                    let mut edgeInCycle = edgeId;
                    edgeStack.push(edgeInCycle);
                    edgeInCycle = edgeTo.get(&graph.edge_endpoints(edgeInCycle).unwrap().0).unwrap().unwrap();

                    while graph.edge_endpoints(edgeInCycle).unwrap().1 != target {
                        edgeStack.push(edgeInCycle);
                        edgeInCycle = edgeTo.get(&graph.edge_endpoints(edgeInCycle).unwrap().0).unwrap().unwrap()
                    }

                    while !edgeStack.is_empty() {
                        new_cycle.add_to_path(graph, edgeStack.pop().unwrap());
                    }

                    *cycle = Some(new_cycle);
                }
            }

            onStack.insert(node, false);
        }

        #[test]
        fn has_cycle_test_0() {
            let mut graph: Graph<u32, f64> = Graph::new();
            let mut nodes: Vec<NodeIndex> = Vec::new();
            for i in 0..5 {
                nodes.push(graph.add_node(i));
            }

            graph.add_edge(nodes[0], nodes[1], 1.0);
            graph.add_edge(nodes[1], nodes[2], 1.0);
            graph.add_edge(nodes[2], nodes[3], 1.0);
            graph.add_edge(nodes[3], nodes[4], 1.0);
            graph.add_edge(nodes[4], nodes[1], 1.0);
            graph.add_edge(nodes[2], nodes[4], 1.0);

            let (cycle_found, cycle) = has_cycle(&graph);
            logObject("cycle0: ", &cycle.unwrap().nodes());

            assert!(cycle_found);
        }

        // Test has_cycle() on a DAG, should not find a cycle.
        #[test]
        fn has_cycle_test_1() {
            let mut graph: Graph<u32, f64> = Graph::new();
            let mut nodes: Vec<NodeIndex> = Vec::new();
            for i in 0..8 {
                nodes.push(graph.add_node(i));
            }

            graph.add_edge(nodes[5], nodes[4], 0.35);
            graph.add_edge(nodes[4], nodes[7], 0.37);
            graph.add_edge(nodes[5], nodes[7], 0.28);
            graph.add_edge(nodes[5], nodes[1], 0.32);
            graph.add_edge(nodes[4], nodes[0], 0.38);
            graph.add_edge(nodes[0], nodes[2], 0.26);
            graph.add_edge(nodes[3], nodes[7], 0.39);
            graph.add_edge(nodes[1], nodes[3], 0.29);
            graph.add_edge(nodes[7], nodes[2], 0.34);
            graph.add_edge(nodes[6], nodes[2], 0.40);
            graph.add_edge(nodes[3], nodes[6], 0.52);
            graph.add_edge(nodes[6], nodes[0], 0.58);
            graph.add_edge(nodes[6], nodes[4], 0.93);

            let (cycle_found, _cycle) = has_cycle(&graph);
            assert!(!cycle_found);
        }

        #[test]
        fn has_cycle_test_2() {
            let mut graph: Graph<u32, f64> = Graph::new();
            let mut nodes: Vec<NodeIndex> = Vec::new();
            for i in 0..8 {
                nodes.push(graph.add_node(i));
            }

            graph.add_edge(nodes[4], nodes[5], 0.35);
            graph.add_edge(nodes[5], nodes[4], 0.35);
            graph.add_edge(nodes[4], nodes[7], 0.37);
            graph.add_edge(nodes[5], nodes[7], 0.28);
            graph.add_edge(nodes[7], nodes[5], 0.28);
            graph.add_edge(nodes[5], nodes[1], 0.32);
            graph.add_edge(nodes[0], nodes[4], 0.38);
            graph.add_edge(nodes[0], nodes[2], 0.26);
            graph.add_edge(nodes[7], nodes[3], 0.39);
            graph.add_edge(nodes[1], nodes[3], 0.29);
            graph.add_edge(nodes[2], nodes[7], 0.34);

            graph.add_edge(nodes[6], nodes[2], -1.20);
            graph.add_edge(nodes[3], nodes[6], 0.52);
            graph.add_edge(nodes[6], nodes[0], -1.40);
            graph.add_edge(nodes[6], nodes[4], -1.25);

            let (cycle_found, cycle) = has_cycle(&graph);
            logObject("cycle2: ", &cycle.unwrap().nodes());
            assert!(cycle_found);
        }
    }
}