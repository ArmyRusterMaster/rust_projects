use std::{env, net::SocketAddr};

#[derive(Clone, Debug)]
pub enum Backend {
    Memory,
    Sqlite { database_url: String },
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub addr: SocketAddr,
    pub backend: Backend,
    pub max_inflight_requests: usize,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let addr = env::var("AUTH_BIND_ADDR")
            .ok()
            .unwrap_or_else(|| "0.0.0.0:3000".to_owned())
            .parse::<SocketAddr>()
            .map_err(|e| format!("AUTH_BIND_ADDR: {e}"))?;

        let backend = match env::var("AUTH_BACKEND")
            .ok()
            .unwrap_or_else(|| "memory".to_owned())
            .as_str()
        {
            "memory" => Backend::Memory,
            "sqlite" => {
                let database_url = env::var("AUTH_SQLITE_URL")
                    .ok()
                    .unwrap_or_else(|| "sqlite://auth.db".to_owned());
                Backend::Sqlite { database_url }
            }
            other => return Err(format!("unsupported AUTH_BACKEND: {other}")),
        };

        let max_inflight_requests = env::var("AUTH_MAX_INFLIGHT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(256);

        Ok(Self {
            addr,
            backend,
            max_inflight_requests,
        })
    }
}
