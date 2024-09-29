use super::graph::{
    find_cycles, get_all_negative_cycles_0, get_all_negative_cycles_1,
    get_all_negative_cycles_for_source, get_negative_cycle_for_source_quick,
    get_negative_cycle_quick, has_cycle,
};
use petgraph::{graph::Graph, prelude::NodeIndex};

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
    assert!(cycle_found);
}

#[test]
fn find_cycles_test_0() {
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

    let cycles = find_cycles(&graph);
    assert!(cycles.len() == 2);
}

// Test has_cycle() on a DAG, should not find a cycle.
#[test]
fn find_cycles_test_1() {
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

    let cycles = find_cycles(&graph);
    assert!(cycles.is_empty());
}

#[test]
fn find_cycles_test_2() {
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

    let cycles = find_cycles(&graph);
    assert!(cycles.len() == 15);
}

#[test]
fn get_negative_cycle_for_source_quick_test_0() {
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

    let (negative_cycle_found, cycle) = get_negative_cycle_for_source_quick(&graph, nodes[0]);
    assert!(!negative_cycle_found);
    assert!(cycle.is_none());

    let (negative_cycle_found, cycle) = get_negative_cycle_quick(&graph);
    assert!(!negative_cycle_found);
    assert!(cycle.is_none());
}

// Test has_cycle() on a DAG, should not find a cycle.
#[test]
fn get_negative_cycle_for_source_quick_test_1() {
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

    let (negative_cycle_found, cycle) = get_negative_cycle_for_source_quick(&graph, nodes[0]);
    assert!(!negative_cycle_found);
    assert!(cycle.is_none());
    let (negative_cycle_found, cycle) = get_negative_cycle_quick(&graph);
    assert!(!negative_cycle_found);
    assert!(cycle.is_none());
}

#[test]
fn get_negative_cycle_for_source_quick_test_2() {
    let mut graph: Graph<u32, f64> = Graph::new();
    let mut nodes: Vec<NodeIndex> = Vec::new();
    for i in 0..8 {
        nodes.push(graph.add_node(i));
    }

    graph.add_edge(nodes[4], nodes[5], 0.35);
    graph.add_edge(nodes[5], nodes[4], -0.66);
    graph.add_edge(nodes[4], nodes[7], 0.37);
    graph.add_edge(nodes[5], nodes[7], 0.28);
    graph.add_edge(nodes[7], nodes[5], 0.28);
    graph.add_edge(nodes[5], nodes[1], 0.32);
    graph.add_edge(nodes[0], nodes[4], 0.38);
    graph.add_edge(nodes[0], nodes[2], 0.26);
    graph.add_edge(nodes[7], nodes[3], 0.39);
    graph.add_edge(nodes[1], nodes[3], 0.29);
    graph.add_edge(nodes[2], nodes[7], 0.34);
    graph.add_edge(nodes[6], nodes[2], 0.40);
    graph.add_edge(nodes[3], nodes[6], 0.52);
    graph.add_edge(nodes[6], nodes[0], 0.58);
    graph.add_edge(nodes[6], nodes[4], 0.93);

    let (negative_cycle_found, cycle) = get_negative_cycle_for_source_quick(&graph, nodes[0]);
    assert!(negative_cycle_found);
    assert!(cycle.unwrap().nodes().len() == 3);
    let (negative_cycle_found, cycle) = get_negative_cycle_quick(&graph);
    assert!(negative_cycle_found);
    assert!(cycle.unwrap().nodes().len() == 3);
}

#[test]
fn get_all_negative_cycles_test_0() {
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

    let cycles = get_all_negative_cycles_0(&graph);
    assert!(cycles.is_empty());
    let cycles = get_all_negative_cycles_1(&graph);
    assert!(cycles.is_empty());
}

// Test has_cycle() on a DAG, should not find a cycle.
#[test]
fn get_all_negative_cycles_test_1() {
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

    let cycles = get_all_negative_cycles_0(&graph);
    assert!(cycles.is_empty());
    let cycles = get_all_negative_cycles_1(&graph);
    assert!(cycles.is_empty());
}

#[test]
fn get_all_negative_cycles_test_2() {
    let mut graph: Graph<u32, f64> = Graph::new();
    let mut nodes: Vec<NodeIndex> = Vec::new();
    for i in 0..8 {
        nodes.push(graph.add_node(i));
    }

    graph.add_edge(nodes[4], nodes[5], 0.35);
    graph.add_edge(nodes[5], nodes[4], -0.66);
    graph.add_edge(nodes[4], nodes[7], 0.37);
    graph.add_edge(nodes[5], nodes[7], 0.28);
    graph.add_edge(nodes[7], nodes[5], 0.28);
    graph.add_edge(nodes[5], nodes[1], 0.32);
    graph.add_edge(nodes[0], nodes[4], 0.38);
    graph.add_edge(nodes[0], nodes[2], 0.26);
    graph.add_edge(nodes[7], nodes[3], 0.39);
    graph.add_edge(nodes[1], nodes[3], 0.29);
    graph.add_edge(nodes[2], nodes[7], 0.34);
    graph.add_edge(nodes[6], nodes[2], 0.40);
    graph.add_edge(nodes[3], nodes[6], 0.52);
    graph.add_edge(nodes[6], nodes[0], 0.58);
    graph.add_edge(nodes[6], nodes[4], 0.93);

    let cycles = get_all_negative_cycles_0(&graph);
    assert!(cycles.len() == 2);
    let cycles = get_all_negative_cycles_1(&graph);
    assert!(cycles.len() == 2);
}
