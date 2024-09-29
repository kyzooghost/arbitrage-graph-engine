// TODO - How to serialize and deserialize message to find appropriate message handler?
use dotenv::dotenv;
use std::fs;
use zeromq::*;

// struct ArbitrageResult {
    // time
    // array of results
// }

pub struct RequestHandler {}

impl RequestHandler {
    pub async fn listen_and_serve() -> Result<(), Box<dyn std::error::Error>> {
        dotenv().ok();
        let ipc_endpoint = std::env::var("IPC_ENDPOINT").unwrap();
        let ipc_filepath = ipc_endpoint[6..].to_string();
        println!("Preparing IPC endpoint at {}", ipc_filepath);
        if fs::metadata(&ipc_filepath).is_ok() {
            fs::remove_file(&ipc_filepath)?;
            println!("Existing IPC file at {}...removing", ipc_filepath);
        }

        println!("Starting API server");
        let mut socket = zeromq::ReqSocket::new();
        socket.bind(&ipc_endpoint).await?;

        loop {
            println!("Sup");
            let mut repl: String = socket.recv().await?.try_into()?;
            println!("Sup2");
            dbg!(&repl);
            repl.push_str(" Reply");
            socket.send(repl.into()).await?;
        }
    }
}