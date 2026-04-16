use anyhow::{anyhow, Result};
use ipnet::IpNet;
use std::net::IpAddr;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

/// Главная структура для подготовки целей сканирования
pub struct TargetResolver {
    resolver: TokioAsyncResolver,
}

impl TargetResolver {
    pub fn new() -> Result<Self> {
        // Используем системные настройки DNS или Google (8.8.8.8)
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );
        Ok(Self { resolver })
    }

    /// Разбирает строку (домен, IP или CIDR) и возвращает список IP-адресов
    pub async fn resolve(&self, target: &str) -> Result<Vec<IpAddr>> {
        // 1. Проверяем, не является ли ввод CIDR-нотацией (например, 192.168.1.0/24)
        if let Ok(net) = target.parse::<IpNet>() {
            return Ok(net.hosts().collect());
        }

        // 2. Проверяем, не является ли ввод одиночным IP-адресом
        if let Ok(ip) = target.parse::<IpAddr>() {
            return Ok(vec![ip]);
        }

        // 3. Если это домен (например, google.com), резолвим его через DNS
        match self.resolver.lookup_ip(target).await {
            Ok(lookup) => {
                let ips: Vec<IpAddr> = lookup.iter().collect();
                if ips.is_empty() {
                    return Err(anyhow!("Домен найден, но IP-адреса отсутствуют"));
                }
                Ok(ips)
            }
            Err(e) => Err(anyhow!("Ошибка резолвинга домена '{}': {}", target, e)),
        }
    }

    /// Вспомогательная функция для парсинга портов (например, "80,443,1000-2000")
    pub fn parse_ports(ports_str: &str) -> Vec<u16> {
        let mut ports = Vec::new();
        for part in ports_str.split(',') {
            if part.contains('-') {
                let bounds: Vec<&str> = part.split('-').collect();
                if let (Ok(start), Ok(end)) = (bounds[0].parse::<u16>(), bounds[1].parse::<u16>()) {
                    ports.extend(start..=end);
                }
            } else if let Ok(port) = part.parse::<u16>() {
                ports.push(port);
            }
        }
        ports.sort();
        ports.dedup();
        ports
    }
}

