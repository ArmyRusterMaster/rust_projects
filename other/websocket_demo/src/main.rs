mod server;
mod client;

use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    use std::env;
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [server|client]", args[0]);
        return;
    }

    match args[1].as_str() {
        "server" => server::start_server(),
        "client" => client::start_client().await,
        _ => eprintln!("Unknown command: {}", args[1]),
    }
}

