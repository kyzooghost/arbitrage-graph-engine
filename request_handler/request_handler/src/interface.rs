// TODO - How to serialize and deserialize message to find appropriate message handler?

// TODO - Need to add metadata to the path, not just exchange rate but protocol + blockchain
struct ArbitrageResult {

    // time
    // array of results
    // 
}

trait RequestHandler {
    // Start server to listen for messages from external processes
    fn listen_and_serve(&self);

    // Update graph data structure
    // Up to the client to provide unique string ID for nodes
    // add_node if not present
    // Will add bidrectional edge between n0 and n1
    // True if graph updated, False if not (not more 'extreme' than existing edges)
    fn upsert_path(&self, n0: &str, n1: &str, n0_to_n1_rate: f64) -> bool;

    // fn scan_arbitrages(&self) 
}