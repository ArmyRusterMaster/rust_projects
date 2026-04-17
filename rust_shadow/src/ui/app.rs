pub struct ConnInfo {
    pub ip: String,
    pub domain: String,
    pub protocol: String,
    pub data_transferred: u64,
}

pub struct App {
    pub should_quit: bool,
    pub connections: Vec<ConnInfo>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            // Тестовые данные для проверки отрисовки
            connections: vec![
                ConnInfo {
                    ip: "1.1.1.1".into(),
                    domain: "cloudflare-dns.com".into(),
                    protocol: "DNS/TLS".into(),
                    data_transferred: 102456,
                },
                ConnInfo {
                    ip: "142.250.185.78".into(),
                    domain: "google.com".into(),
                    protocol: "HTTPS".into(),
                    data_transferred: 2500400,
                },
            ],
        }
    }

    pub fn on_tick(&mut self) {
        // Логика обновления будет здесь
    }
}
