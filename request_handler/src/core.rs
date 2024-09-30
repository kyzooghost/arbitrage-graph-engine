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
        if fs::metadata(&ipc_filepath).is_ok() {
            fs::remove_file(&ipc_filepath)?;
            println!("Removing existing IPC file at {}", ipc_filepath);
        }

        let mut socket = zeromq::RepSocket::new();
        socket.bind(&ipc_endpoint).await?;
        println!("API server started at {}", ipc_filepath);
        loop {
            let mut repl: String = socket.recv().await?.try_into()?;
            dbg!(&repl);
            repl.push_str(" Reply");
            socket.send(repl.into()).await?;
        }
    }
}
