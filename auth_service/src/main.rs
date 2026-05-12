#[tokio::main]
async fn main() {
    let server = auth_service::server::Server::new(auth_service::server::ServerConfig::default());
    if let Err(error) = server.run().await {
        eprintln!("server failed: {error}");
    }
}
