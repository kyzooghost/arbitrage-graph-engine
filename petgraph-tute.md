```rust
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
```