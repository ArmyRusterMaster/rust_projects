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
    pub request_limit_per_sec: u64,
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

        let request_limit_per_sec = env::var("AUTH_REQUESTS_PER_SEC")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(50);

        Ok(Self {
            addr,
            backend,
            request_limit_per_sec,
        })
    }
}
