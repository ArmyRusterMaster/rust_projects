use std::net::SocketAddr;

use crate::http;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub addr: SocketAddr,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], 3000)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Server {
    config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn run(self) -> std::io::Result<()> {
        let app = http::build_router();
        let listener = tokio::net::TcpListener::bind(self.config.addr).await?;

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        let _ = signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
