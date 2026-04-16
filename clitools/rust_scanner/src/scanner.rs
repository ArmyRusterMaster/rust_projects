use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};

pub struct PortScanner {
    timeout: Duration,
    max_concurrent: usize,
}

impl PortScanner {
    pub fn new(timeout_ms: u64, max_concurrent: usize) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms),
            max_concurrent,
        }
    }

    pub async fn scan(
        &self,
        ips: Vec<IpAddr>,
        ports: Vec<u16>,
        tx: mpsc::UnboundedSender<(IpAddr, u16, bool)>,
    ) {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));

        for ip in ips {
            for port in ports.clone() {
                // Захватываем разрешение ПЕРЕД спавном, чтобы не перегружать память
                let permit = semaphore.clone().acquire_owned().await.expect("Semaphore error");
                let tx_clone = tx.clone();
                let timeout_duration = self.timeout;

                tokio::spawn(async move {
                    let is_open = match timeout(timeout_duration, TcpStream::connect((ip, port))).await {
                        Ok(Ok(_)) => true,
                        _ => false,
                    };
                    let _ = tx_clone.send((ip, port, is_open));
                    drop(permit); // Разрешение возвращается в семафор здесь
                });
            }
        }
    }
}

