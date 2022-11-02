pub use graph_mod::*;

mod graph_mod {

    pub mod edge {
        #[derive(Debug)]
        pub struct Edge {
            pub node1: usize,
            node2: usize,
            weight: f64,
        }
    
        impl Edge {
            pub fn new(node1_: usize, node2_: usize, weight_: f64) -> Self {
                Edge {
                    node1: node1_,
                    node2: node2_,
                    weight: weight_
                }
            }
    
            pub fn weight(&self) -> &f64 {
                &self.weight
            }
    
            pub fn from(&self) -> usize {
                self.node1
            }  
    
            pub fn to(&self) -> usize {
                self.node2
            }  
        }
    
        #[test]
        fn edge() {
            let edge = Edge::new(0, 1, 0.1);
            assert_eq!(edge.from(), 0);
            assert_eq!(edge.to(), 1);
            assert_eq!(edge.weight().to_bits(), 0.1_f64.to_bits());
        }
    }

    pub mod digraph {
        use super::edge::Edge;

        #[derive(Debug)]
        pub struct Graph {
            node_count: usize,
            edge_count: usize,
            adjacent: Vec<Vec<Edge>>
        }
    
        impl Graph {
            pub fn new(initial_node_count: usize) -> Self {
                let mut adjacent: Vec<Vec<Edge>> = Vec::new();
    
                for _ in 0..initial_node_count {
                    adjacent.push(Vec::new());
                }
    
                Graph {
                    node_count: initial_node_count,
                    edge_count: 0,
                    adjacent
                }
            }
    
            pub fn node_count(&self) -> &usize {
                &self.node_count
            }
    
            pub fn edge_count(&self) -> &usize {
                &self.edge_count
            }
    
            pub fn outdegree(&self, node: usize) -> usize {
                self.adjacent[node].len()
            }
    
            pub fn add_edge(&mut self, edge: Edge) {
                self.adjacent[edge.from()].push(edge);
                self.edge_count += 1; 
            }
    
            pub fn add_node(&mut self) {
                self.node_count += 1; 
                self.adjacent.push(Vec::new());
            }
    
            pub fn get_adjacent_edges(&self, node: usize) -> &Vec<Edge> {
                &self.adjacent[node]
            }
    
            pub fn get_all_edges(&self) -> Vec<&Edge> {
                let mut edges: Vec<&Edge> = Vec::new();
    
                for node in 0..self.node_count {
                    for edge in self.adjacent[node].iter() {
                        edges.push(edge);
                    }
                }
    
                edges
            }
        }
    
        #[test]
        fn graph() {
            let mut graph = Graph::new(2);
            assert_eq!(graph.node_count(), &2);
            assert_eq!(graph.edge_count(), &0);
            assert_eq!(graph.outdegree(0), 0);
            graph.add_edge(Edge::new(0, 1, 0.1));
            assert_eq!(graph.edge_count(), &1);
            assert_eq!(graph.outdegree(0), 1);
            assert_eq!(graph.outdegree(1), 0);   
            assert_eq!(graph.get_adjacent_edges(0).len(), 1);
            assert_eq!(graph.get_adjacent_edges(0)[0].from(), 0);
            assert_eq!(graph.get_adjacent_edges(0)[0].to(), 1);
            assert_eq!(graph.get_adjacent_edges(0)[0].weight().to_bits(), 0.1_f64.to_bits());
            assert_eq!(graph.get_adjacent_edges(1).len(), 0);
            assert_eq!(graph.get_all_edges().len(), 1);
            assert_eq!(graph.get_all_edges()[0].from(), 0);
            assert_eq!(graph.get_all_edges()[0].to(), 1);
            assert_eq!(graph.get_all_edges()[0].weight().to_bits(), 0.1_f64.to_bits());
        }
    }



}

// #[cfg(test)]
// mod graph_tests {
//     use super::Graph;

// }
