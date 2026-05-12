#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config = auth_service::config::AppConfig::from_env().expect("invalid config");
    let server = auth_service::server::Server::new(config);
    if let Err(error) = server.run().await {
        eprintln!("server failed: {error}");
    }
}
