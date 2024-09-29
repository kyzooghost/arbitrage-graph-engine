use eyre::Result;
use request_handler::core::RequestHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RequestHandler::listen_and_serve().await?;
    Ok(())
}
