pub use graph_mod::*;
use super::utils::logger::log;

mod graph_mod {
    use petgraph::graph::Graph;

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
        use petgraph::prelude::{NodeIndex, EdgeIndex};
        use super::Graph;
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


}