#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config = auth_service::config::AppConfig::from_env().map_err(auth_service::StartupError::Config);
    let Ok(config) = config else {
        eprintln!("startup failed: {:?}", config.err().unwrap());
        std::process::exit(2);
    };
    let server = auth_service::server::Server::new(config);
    if let Err(error) = server.run().await {
        eprintln!("startup failed: {error}");
        std::process::exit(1);
    }
}
