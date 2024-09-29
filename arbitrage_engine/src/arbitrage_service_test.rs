#[cfg(test)]
mod tests {
    use crate::{
        decorated_edge::DecoratedEdge,
        arbitrage_service::{IArbitrageService, ArbitrageService}
    };

    #[test]
    fn test_upsert_path_single_path_success() {
        let mut service: ArbitrageService = ArbitrageService::new();
        let edge = DecoratedEdge {
            weight: 1.0,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let is_updated = service.upsert_path("a", "b", edge);
        assert!(is_updated);
    }

    #[test]
    fn test_upsert_path_duplicate_path_should_return_false() {
        let mut service: ArbitrageService = ArbitrageService::new();
        let edge = DecoratedEdge {
            weight: 1.0,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let duplicated_edge = DecoratedEdge {
            weight: 1.0,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let _ = service.upsert_path("a", "b", edge);
        let is_updated = service.upsert_path("a", "b", duplicated_edge);
        assert!(!is_updated);
    }

    #[test]
    fn test_upsert_path_two_paths_between_same_nodes_success() {
        let mut service: ArbitrageService = ArbitrageService::new();
        let edge_0 = DecoratedEdge {
            weight: 1.0,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let edge_1 = DecoratedEdge {
            weight: 1.0,
            protocol_type: 2,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let _ = service.upsert_path("a", "b", edge_0);
        let is_updated = service.upsert_path("a", "b", edge_1);
        assert!(is_updated);
    }

    #[test]
    fn test_upsert_path_third_edge_between_existing_edges_success() {
        let mut service: ArbitrageService = ArbitrageService::new();
        let edge_0 = DecoratedEdge {
            weight: 0.95,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let edge_1 = DecoratedEdge {
            weight: 1.05,
            protocol_type: 2,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let new_edge = DecoratedEdge {
            weight: 1.0,
            protocol_type: 3,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let _ = service.upsert_path("a", "b", edge_0);
        let _ = service.upsert_path("a", "b", edge_1);
        let is_updated = service.upsert_path("a", "b", new_edge);
        assert!(!is_updated);
    }

    #[test]
    fn test_upsert_path_third_edge_above_existing_edges_success() {
        let mut service: ArbitrageService = ArbitrageService::new();
        let edge_0 = DecoratedEdge {
            weight: 0.95,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let edge_1 = DecoratedEdge {
            weight: 1.05,
            protocol_type: 2,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let new_edge = DecoratedEdge {
            weight: 1.10,
            protocol_type: 3,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let _ = service.upsert_path("a", "b", edge_0);
        let _ = service.upsert_path("a", "b", edge_1);
        let is_updated = service.upsert_path("a", "b", new_edge);
        assert!(is_updated);
    }

    #[test]
    fn test_upsert_path_third_edge_below_existing_edges_success() {
        let mut service: ArbitrageService = ArbitrageService::new();
        let edge_0 = DecoratedEdge {
            weight: 0.95,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let edge_1 = DecoratedEdge {
            weight: 1.05,
            protocol_type: 2,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let new_edge = DecoratedEdge {
            weight: 0.90,
            protocol_type: 3,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };
        let _ = service.upsert_path("a", "b", edge_0);
        let _ = service.upsert_path("a", "b", edge_1);
        let is_updated = service.upsert_path("a", "b", new_edge);
        assert!(is_updated);
    }

    // Use same test conditions as engine_test.get_negative_cycle_for_source_quick_test_2
    #[test]
    fn test_scan_arbitrages_quick_success() {
        let mut service: ArbitrageService = ArbitrageService::new();
        
        let new_decorated_edge = |weight: f64| DecoratedEdge {
            weight,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };

        service.upsert_path("nodes[4]", "nodes[5]", new_decorated_edge(0.35));
        service.upsert_path("nodes[5]", "nodes[4]", new_decorated_edge(-0.66));
        service.upsert_path("nodes[4]", "nodes[7]", new_decorated_edge(0.37));
        service.upsert_path("nodes[5]", "nodes[7]", new_decorated_edge(0.28));
        service.upsert_path("nodes[7]", "nodes[5]", new_decorated_edge(0.28));
        service.upsert_path("nodes[5]", "nodes[1]", new_decorated_edge(0.32));
        service.upsert_path("nodes[0]", "nodes[4]", new_decorated_edge(0.38));
        service.upsert_path("nodes[0]", "nodes[2]", new_decorated_edge(0.26));
        service.upsert_path("nodes[7]", "nodes[3]", new_decorated_edge(0.39));
        service.upsert_path("nodes[1]", "nodes[3]", new_decorated_edge(0.29));
        service.upsert_path("nodes[2]", "nodes[7]", new_decorated_edge(0.34));
        service.upsert_path("nodes[6]", "nodes[2]", new_decorated_edge(0.40));
        service.upsert_path("nodes[3]", "nodes[6]", new_decorated_edge(0.52));
        service.upsert_path("nodes[6]", "nodes[0]", new_decorated_edge(0.58));
        service.upsert_path("nodes[6]", "nodes[4]", new_decorated_edge(0.93));

        assert_eq!(service.node_count(), 8);
        let paths = service.scan_arbitrages_quick();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].nodes.len(), 3);
        assert_eq!(paths[0].edges.len(), 2);
    }

    #[test]
    fn test_scan_arbitrages_quick() {
        let mut service: ArbitrageService = ArbitrageService::new();
        
        let new_decorated_edge = |weight: f64| DecoratedEdge {
            weight,
            protocol_type: 1,
            node_type: 1,
            pool_address: "".to_string(),
            data: "".to_string()
        };

        service.upsert_path("nodes[4]", "nodes[5]", new_decorated_edge(0.35));
        service.upsert_path("nodes[5]", "nodes[4]", new_decorated_edge(-0.66));
        service.upsert_path("nodes[4]", "nodes[7]", new_decorated_edge(0.37));
        service.upsert_path("nodes[5]", "nodes[7]", new_decorated_edge(0.28));
        service.upsert_path("nodes[7]", "nodes[5]", new_decorated_edge(0.28));
        service.upsert_path("nodes[5]", "nodes[1]", new_decorated_edge(0.32));
        service.upsert_path("nodes[0]", "nodes[4]", new_decorated_edge(0.38));
        service.upsert_path("nodes[0]", "nodes[2]", new_decorated_edge(0.26));
        service.upsert_path("nodes[7]", "nodes[3]", new_decorated_edge(0.39));
        service.upsert_path("nodes[1]", "nodes[3]", new_decorated_edge(0.29));
        service.upsert_path("nodes[2]", "nodes[7]", new_decorated_edge(0.34));
        service.upsert_path("nodes[6]", "nodes[2]", new_decorated_edge(0.40));
        service.upsert_path("nodes[3]", "nodes[6]", new_decorated_edge(0.52));
        service.upsert_path("nodes[6]", "nodes[0]", new_decorated_edge(0.58));
        service.upsert_path("nodes[6]", "nodes[4]", new_decorated_edge(0.93));

        assert_eq!(service.node_count(), 8);
        let paths = service.scan_arbitrages();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].nodes.len(), 3);
        assert_eq!(paths[0].edges.len(), 2);
    }
}