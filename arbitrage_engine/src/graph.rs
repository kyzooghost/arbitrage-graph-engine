#![allow(dead_code, unused, nonstandard_style)]

pub use graph_mod::*;

mod graph_mod {
    use petgraph::{
        graph::{Graph, EdgeReference},
        prelude::{EdgeIndex, NodeIndex},
        visit::{EdgeRef},
        Direction::Outgoing,
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
        use super::{EdgeIndex, Graph, NodeIndex};
        use std::{
            marker::PhantomData,
            cmp::Ordering
        };

        // Choosing to store vector of indexes, rather than using a recursive type as we did in Java. Should be ok if the vectors don't grow too large.
        #[derive(Debug)]
        pub struct Path<N: Clone> {
            weight: f64,
            edges: Vec<EdgeIndex>,
            nodes: Vec<NodeIndex>,
            node_type: PhantomData<N>,
        }

        impl<N: Clone> Path<N> {
            pub fn new(source_node: NodeIndex) -> Self {
                Path {
                    weight: 0.0_f64,
                    edges: Vec::new(),
                    nodes: vec![source_node],
                    node_type: PhantomData,
                }
            }

            pub fn add_to_path(&mut self, graph: &Graph<N, f64>, edge: EdgeIndex) {
                assert_eq!(
                    &graph.edge_endpoints(edge).unwrap().0,
                    self.nodes.last().unwrap(),
                    "Edge does not extend from existing path"
                );
                self.weight += graph.edge_weight(edge).unwrap();
                self.edges.push(edge);
                self.nodes.push(graph.edge_endpoints(edge).unwrap().1);
            }

            pub fn weight(&self) -> f64 {
                self.weight
            }

            pub fn edges(&self) -> Vec<EdgeIndex> {
                let mut vec: Vec<EdgeIndex> = Vec::new();
                for edge in self.edges.iter() {
                    vec.push(*edge);
                }
                vec
            }

            pub fn nodes(&self) -> Vec<NodeIndex> {
                let mut vec: Vec<NodeIndex> = Vec::new();
                for node in self.nodes.iter() {
                    vec.push(*node);
                }
                vec
            }

            pub fn length(&self) -> usize {
                self.nodes.len()
            }
        }

        impl<N: Clone> PartialEq for Path<N> {
            fn eq(&self, other: &Self) -> bool {
                self.weight.to_bits() == other.weight.to_bits()
            }
        }

        impl<N: Clone> Eq for Path<N> {}

        impl<N: Clone> PartialOrd for Path<N> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<N: Clone> Ord for Path<N> {
            fn cmp(&self, other: &Self) -> Ordering {
                if self.weight < other.weight {
                    Ordering::Less
                } else if self.weight > other.weight {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
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
        use super::{
            logObject, 
            logText, 
            path::Path, 
            EdgeIndex, 
            EdgeRef, 
            EdgeReference,
            Graph, 
            NodeIndex, 
            Outgoing,
        };

        use std::{
            collections::{HashMap, HashSet, VecDeque},
        };

        // Method 2 for obtaining all negative cycles, sorted from most negative to least.
        // Uses find_cycles within Bellman_Ford, cycles_found() occurs on graphs with less noise, however already O(V) factor for outer loop in function body and duplicate work from encountering the same path.
        pub fn get_all_negative_cycles_1<N: Clone>(graph: &Graph<N, f64>) -> Vec<Path<N>> {
            let mut paths:Vec<Path<N>> = Vec::new();
            for node in graph.node_indices() {
                let mut cycles_found = get_all_negative_cycles_for_source(graph, node);
                paths.append(&mut cycles_found);
            }
            let mut negative_paths: Vec<Path<N>> = paths.into_iter().filter(|path| path.weight() < 0.0).collect();
            negative_paths.sort_unstable();
            negative_paths.dedup_by(|a, b| a.weight().to_bits() == b.weight().to_bits());
            negative_paths
        }

        // Method 1 for obtaining all negative cycles, sorted from most negative to least.
        // Uses find_cycles() on unfiltered graph, may suffer noise in the graph.
        pub fn get_all_negative_cycles_0<N: Clone>(graph: &Graph<N, f64>) -> Vec<Path<N>> {
            let paths = find_cycles(graph);
            let mut negative_paths: Vec<Path<N>> = paths.into_iter().filter(|path| path.weight() < 0.0).collect();
            negative_paths.sort_unstable();
            negative_paths
        }

        // Attempts get_negative_cycle_for_source_quick for all nodes, stops if it finds a negative cycle
        pub fn get_negative_cycle_quick<N: Clone>(graph: &Graph<N, f64>) -> (bool, Option<Path<N>>) {
            for node in graph.node_indices() {
                let (negative_cycle_found, cycle) = get_negative_cycle_for_source_quick(graph, node);
                if negative_cycle_found {
                    return (negative_cycle_found, cycle);
                }
            }
            (false, None)
        }

        // Modified queue-based Bellman-Ford algorithm. Only difference with get_negative_cycle_for_source_quick is that we call find_cycles() after a successful has_cycle() call.
        fn get_all_negative_cycles_for_source<N: Clone>(graph: &Graph<N, f64>, source: NodeIndex) -> Vec<Path<N>> {
            // Node => Weight of current shortest path from source.
            let mut dist: HashMap<NodeIndex, f64> = HashMap::new();
            // Node => Edge in current shortest path with node as target_node.
            let mut edgeTo: HashMap<NodeIndex, Option<EdgeReference<f64>>> = HashMap::new();
            let mut queue: VecDeque<NodeIndex> = VecDeque::new();
            let mut on_queue: HashMap<NodeIndex, bool>= HashMap::new();
            // Counter of relax operations
            let mut counter = 0;

            for node in graph.node_indices() {
                edgeTo.insert(node, None);
                dist.insert(node, f64::MAX);
                on_queue.insert(node, false);
            }

            queue.push_back(source);
            on_queue.insert(source, true);
            dist.insert(source, 0.0);

            while !queue.is_empty() {
                let current_node = queue.pop_front().unwrap();
                on_queue.insert(source, false);

                // Iterate through all neighbours
                for edge in graph.edges_directed(current_node, Outgoing) {
                    let weight = edge.weight();
                    let target_node = edge.target();

                    // Relax operation
                    if *dist.get(&target_node).unwrap() > dist.get(&current_node).unwrap() + weight {
                        *dist.get_mut(&target_node).unwrap() = dist.get(&current_node).unwrap() + weight;
                        *edgeTo.get_mut(&target_node).unwrap() = Some(edge);
                        counter += 1;

                        if !on_queue.get(&target_node).unwrap() {
                            queue.push_back(target_node);
                            on_queue.insert(target_node, true);
                        }

                        // Check for cycle every V times we call relax.
                        if counter % graph.node_count() == 0 {
                            // Construct current SPT from edgeTo collection
                            let mut spt: Graph<N, f64> = Graph::new();

                            // Zzz need to implement N: Clone trait just for this one line
                            // I guess the issue is that `graph` owns N, but to make another graph with the same nodes we need to `copy` the nodes over. So we need to tell the comiler that N is a type that can be safely deep cloned.
                            for node in graph.node_indices() {
                                spt.add_node(graph.node_weight(node).unwrap().clone());
                            }

                            for node in graph.node_indices() {
                                if let Some(spt_edge) = edgeTo.get(&node).unwrap() {
                                    let spt_edge_weight = spt_edge.weight();
                                    let spt_edge_source = spt_edge.source();
                                    spt.add_edge(spt_edge_source, node, *spt_edge_weight);
                                }
                            }

                            let (cycle_found, spt_cycle) = has_cycle(&spt);
                            if cycle_found {
                                return find_cycles(&spt);
                            }
                        }
                    }
                }
            }

            // Emptied queue without finding cycle
            vec![]
        }

        // Modified queue-based Bellman-Ford algorithm. O(E) practically, O(V * E) theoretically.
        fn get_negative_cycle_for_source_quick<N: Clone>(graph: &Graph<N, f64>, source: NodeIndex) -> (bool, Option<Path<N>>) {
            // Node => Weight of current shortest path from source.
            let mut dist: HashMap<NodeIndex, f64> = HashMap::new();
            // Node => Edge in current shortest path with node as target_node.
            let mut edgeTo: HashMap<NodeIndex, Option<EdgeReference<f64>>> = HashMap::new();
            let mut queue: VecDeque<NodeIndex> = VecDeque::new();
            let mut on_queue: HashMap<NodeIndex, bool>= HashMap::new();
            // Counter of relax operations
            let mut counter = 0;

            for node in graph.node_indices() {
                edgeTo.insert(node, None);
                dist.insert(node, f64::MAX);
                on_queue.insert(node, false);
            }

            queue.push_back(source);
            on_queue.insert(source, true);
            dist.insert(source, 0.0);

            while !queue.is_empty() {
                let current_node = queue.pop_front().unwrap();
                on_queue.insert(source, false);

                // Iterate through all neighbours
                for edge in graph.edges_directed(current_node, Outgoing) {
                    let weight = edge.weight();
                    let target_node = edge.target();

                    // Relax operation
                    if *dist.get(&target_node).unwrap() > dist.get(&current_node).unwrap() + weight {
                        *dist.get_mut(&target_node).unwrap() = dist.get(&current_node).unwrap() + weight;
                        *edgeTo.get_mut(&target_node).unwrap() = Some(edge);
                        counter += 1;

                        if !on_queue.get(&target_node).unwrap() {
                            queue.push_back(target_node);
                            on_queue.insert(target_node, true);
                        }

                        // Check for cycle every V times we call relax.
                        if counter % graph.node_count() == 0 {
                            // Construct current SPT from edgeTo collection
                            let mut spt: Graph<N, f64> = Graph::new();

                            // Zzz need to implement N: Clone trait just for this one line
                            // I guess the issue is that `graph` owns N, but to make another graph with the same nodes we need to `copy` the nodes over. So we need to tell the comiler that N is a type that can be safely deep cloned.
                            for node in graph.node_indices() {
                                spt.add_node(graph.node_weight(node).unwrap().clone());
                            }

                            for node in graph.node_indices() {
                                if let Some(spt_edge) = edgeTo.get(&node).unwrap() {
                                    let spt_edge_weight = spt_edge.weight();
                                    let spt_edge_source = spt_edge.source();
                                    spt.add_edge(spt_edge_source, node, *spt_edge_weight);
                                }
                            }

                            let (cycle_found, spt_cycle) = has_cycle(&spt);
                            
                            if cycle_found { 
                                return (cycle_found, spt_cycle); 
                            }
                        }
                    }
                }
            }

            // Emptied queue without finding cycle
            (false, None)
        }

        // DFS algorithm to determine if a cycle exists in a graph, linear time algorithm: O(V + E).
        // Returns tuple
        // tuple.0 (bool): false if no cycle found, true if cycle present.
        // tuple.1 (Option<Path<N>>): None if no cycle found, Path representing cycle if cycle found.
        fn has_cycle<N: Clone>(graph: &Graph<N, f64>) -> (bool, Option<Path<N>>) {
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
                    _has_cycle_dfs(
                        graph,
                        node,
                        &mut visited,
                        &mut edgeTo,
                        &mut onStack,
                        &mut cycle,
                    );
                }
            }

            match cycle {
                None => (false, None),
                Some(discovered_cycle) => (true, Some(discovered_cycle)),
            }
        }

        fn _has_cycle_dfs<N: Clone>(
            graph: &Graph<N, f64>,
            node: NodeIndex,
            visited: &mut HashMap<NodeIndex, bool>,
            edgeTo: &mut HashMap<NodeIndex, Option<EdgeIndex>>,
            onStack: &mut HashMap<NodeIndex, bool>,
            cycle: &mut Option<Path<N>>,
        ) {
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
                // If target node is on stack, we have found a cycle
                } else if *onStack.get(&target).unwrap() {
                    // We have an issue that edgeTo gives EdgeIndex type, whereas we need type N for Path
                    // Also Path needs to start from the first node, whereas we can only get our cycles backwards by unpopping the stack.
                    let mut new_cycle: Path<N> = Path::new(target);
                    let mut edgeStack: Vec<EdgeIndex> = vec![edgeId];

                    // Fill edgeStack with cycle edges, going backwards                    
                    loop {
                        let most_recent_edge = &graph.edge_endpoints(*edgeStack.last().unwrap()).unwrap();
                        let most_recent_edge_from = most_recent_edge.0;

                        if most_recent_edge_from == target {
                            break;
                        }

                        let previous_edge = edgeTo.get(&most_recent_edge_from).unwrap().unwrap();
                        edgeStack.push(previous_edge);
                    }

                    while !edgeStack.is_empty() {
                        new_cycle.add_to_path(graph, edgeStack.pop().unwrap());
                    }

                    *cycle = Some(new_cycle);
                }
            }

            onStack.insert(node, false);
        }

        // Intuitively this is O((E + V) * C), since it's a DFS-style approach and it 'unwinds' whenever a cycle is found.
        // In our application, our initial graph will be a complete graph hence we will waste a lot of 'C's on noise.
        // So we hope to filter our graph down using modified Bellman-Ford prior to using this function.
        fn find_cycles<N: Clone>(graph: &Graph<N, f64>) -> Vec<Path<N>> {
            // Collections of cycles
            let mut cycles: Vec<Path<N>> = Vec::new();
            // Node => isBlocked
            let mut blocked: HashMap<NodeIndex, bool> = HashMap::new();
            // source_node => edges, represented by vector of destination_nodes
            let mut edges: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
            // source_node => blocked edges, represented by destination node?
            let mut blocked_edges: HashMap<NodeIndex, HashSet<NodeIndex>> = HashMap::new();
            // Unsure of what we use this stack for
            let mut stack: Vec<NodeIndex> = Vec::new();
            // Set to keep track of which nodes we have call circuit for
            let mut circuited_nodes: HashSet<NodeIndex> = HashSet::new();
            // Collection of all nodes we have yet to call circuit on
            let mut uncircuited_nodes: Vec<NodeIndex> = Vec::new();

            // Initialize data structures
            for node in graph.node_indices() {
                blocked.insert(node, false);
                blocked_edges.insert(node, HashSet::new());
                uncircuited_nodes.push(node);

                let mut neighbors: Vec<NodeIndex> = Vec::new();
                for target_node in graph.neighbors(node) {
                    neighbors.push(target_node);
                }
                edges.insert(node, neighbors);
            }

            while !uncircuited_nodes.is_empty() {
                let start: NodeIndex = uncircuited_nodes.pop().unwrap();

                // Reset all blocked markers for nodes and edges
                for node in graph.node_indices() {
                    blocked.insert(node, false);
                    blocked_edges.get_mut(&node).unwrap().clear();
                }

                _find_cycles_circuit(
                    start,
                    start,
                    graph,
                    &mut blocked,
                    &mut edges,
                    &mut blocked_edges,
                    &mut cycles,
                    &mut stack,
                    &mut circuited_nodes,
                );
                circuited_nodes.insert(start);
            }

            cycles
        }

        fn _find_cycles_circuit<N: Clone>(
            // Node we are currently visiting with circuit
            circuit_node: NodeIndex,
            // Node we visited in the first circuit call (should be bottom of stack?)
            start_node: NodeIndex,
            graph: &Graph<N, f64>,
            blocked: &mut HashMap<NodeIndex, bool>,
            edges: &HashMap<NodeIndex, Vec<NodeIndex>>,
            blocked_edges: &mut HashMap<NodeIndex, HashSet<NodeIndex>>,
            cycles: &mut Vec<Path<N>>,
            stack: &mut Vec<NodeIndex>,
            circuited_nodes: &mut HashSet<NodeIndex>,
        ) -> bool {
            let mut is_circuit_found = false;
            // Keeping track of what is on the recursion stack.
            stack.push(circuit_node);
            // Not only do we keep track on the recursion stack, but also put a temporary 'blocked' marker on it? Not a permanent 'visited' marker.
            blocked.insert(circuit_node, true);

            // Iterate through every edge with node == source,
            for target_node in edges.get(&circuit_node).unwrap() {
                // If we have already invoked circuit for this node, skip
                if circuited_nodes.contains(target_node) {
                    continue;
                }

                // We have found a circuit, if we have found our start_node again
                // TO-DO, can we replace start_node with bottom of the stack?
                if target_node == &start_node {
                    let mut cycle: Path<N> = Path::new(start_node);
                    for cycle_node in stack.iter() {
                        if cycle_node != &start_node {
                            // Two issues here - 1.) Could have O(1) time here if we refactored, but it's O(e') where e' is edges connected to a instead
                            let edge = graph
                                .find_edge(*cycle.nodes().last().unwrap(), *cycle_node)
                                .unwrap();
                            cycle.add_to_path(graph, edge);
                        }
                    }

                    // Cycle contains all nodes in the cycle, but is missing an edge.
                    let final_edge = graph.find_edge(*cycle.nodes().last().unwrap(), *cycle.nodes().first().unwrap()).unwrap();
                    cycle.add_to_path(graph, final_edge);

                    cycles.push(cycle);

                    is_circuit_found = true;
                // Else if target_node isn't blocked && recursive call of circuit on target_node returns true
                // There is only one condition to return true, if circuit has been found
                } else if !blocked.get(target_node).unwrap()
                    && _find_cycles_circuit(
                        *target_node,
                        start_node,
                        graph,
                        blocked,
                        edges,
                        blocked_edges,
                        cycles,
                        stack,
                        circuited_nodes,
                    )
                {
                    is_circuit_found = true;
                }
            }

            // If we have found a circuit, unblock the node?
            if is_circuit_found {
                _find_cycles_unblock::<N>(circuit_node, blocked, blocked_edges);
            // Iterate through every edge with node == source, again.
            } else {
                for target_node in edges.get(&circuit_node).unwrap() {
                    // Skip if we have already circuited this node.
                    if circuited_nodes.contains(target_node) {
                        continue;
                    }
                    // Hmmm, but this is backwards to edges? It is indexed by target_node?
                    // But we don't use this blocked_edges collection anywhere? We don't use it in any conditional?
                    // So any edge leading into the circuited node, needs to be marked as blocked.
                    if !blocked_edges
                        .get(target_node)
                        .unwrap()
                        .contains(&circuit_node)
                    {
                        blocked_edges
                            .get_mut(target_node)
                            .unwrap()
                            .insert(circuit_node);
                    }
                }
            }

            stack.pop();
            is_circuit_found
        }

        fn _find_cycles_unblock<N>(
            target_node: NodeIndex,
            blocked: &mut HashMap<NodeIndex, bool>,
            blocked_edges: &mut HashMap<NodeIndex, HashSet<NodeIndex>>,
        ) {
            blocked.insert(target_node, false);

            let mut source_nodes_to_unblock: Vec<NodeIndex> = blocked_edges
                .get(&target_node)
                .unwrap()
                .iter()
                .cloned()
                .collect();

            while !source_nodes_to_unblock.is_empty() {
                let source_node = source_nodes_to_unblock.pop().unwrap();

                // Will only call recursive unblock if node is blocked
                // Cannot call recursive unblock on itself, because we have unblocked it at the start of this function
                // So we don't need to worry about mutating the same hashset
                // Should be to move the hashset elements into vector, and clear the hashset (where is the one liner to do that lol)
                // And the implementation for removeFromList() is an O(N) implementation anyway, making unblock an O(N^2) function if we ignore the recursive part. Provided that converting from hashset to vector is an O(N) operation, we have reduced it to an O(N) operation - copy whole hashset, then iterate through single loop for hashset elements
                if *blocked.get(&source_node).unwrap() {
                    _find_cycles_unblock::<N>(source_node, blocked, blocked_edges);
                }
            }

            blocked_edges.get_mut(&target_node).unwrap().clear()
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
    }
}
