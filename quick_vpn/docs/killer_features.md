# Killer features для quick_vpn

Дата: 2026-05-12.

## 1. Hybrid transport engine

Одна платформа, несколько transport backend'ов:

- native QUIC;
- Hysteria-like QUIC;
- external Xray/sing-box;
- future VLESS-lite;
- TCP/TLS fallback.

Почему это killer feature: пользователь не привязан к одному протоколу, а проект показывает зрелую архитектуру.

## 2. Smart diagnostics

Команда:

```text
quick-vpn doctor --profile config.toml
```

Проверяет:

- UDP доступность;
- DNS;
- SNI/cert;
- QUIC handshake;
- MTU;
- route loop;
- TUN permissions;
- backend health;
- clock/time drift;
- firewall hints.

Почему это killer feature: у таких систем главная боль - не протокол, а диагностика.

## 3. Safe external backend adapter

quick_vpn может управлять Xray/sing-box без переписывания всего:

- генерирует config;
- запускает process;
- проверяет health;
- читает stats;
- валидирует profile;
- делает migration между profiles.

Почему это killer feature: быстро дает совместимость с VLESS/REALITY/Hysteria2/TUIC и сохраняет Rust core.

## 4. TUN-first routing engine

Фичи:

- split tunneling;
- domain/IP/process rules;
- direct/block/proxy actions;
- DNS leak guard;
- loop prevention;
- per-app rules на Android в будущем;
- route simulation: показать, куда пойдет конкретный domain/IP.

Почему это killer feature: отличает VPN-платформу от простого proxy.

## 5. Observable tunnel

Метрики:

- RTT;
- packet loss approximation;
- datagram drops;
- reconnects;
- throughput;
- active sessions;
- auth failures;
- MTU errors;
- per-route counters.

Экспорт:

- Prometheus endpoint;
- JSON status;
- compact CLI dashboard.

Почему это killer feature: production-инженеры любят видимость и отладку.

## 6. Obfuscation profiles with honest tradeoffs

Профили:

- `fast`: минимум padding, максимум throughput.
- `balanced`: randomized keepalive + initial padding.
- `stealth-web`: external Xray/Naive-like backend.
- `stealth-udp`: Hysteria/AmneziaWG-like packet size randomization.
- `mobile`: battery-friendly keepalive/reconnect.

Почему это killer feature: пользователь выбирает цель, а не тонет в 50 низкоуровневых настройках.

## 7. Config compatibility layer

Импорт:

- `vless://`
- `ss://`
- `hy2://`
- `tuic://`
- sing-box JSON subset
- Xray JSON subset

Экспорт:

- quick_vpn TOML;
- external backend config;
- share link.

Почему это killer feature: снижает порог входа и делает проект полезным даже до полной собственной реализации всех протоколов.

## 8. Reproducible demo lab

В репозитории:

- `docker compose` lab;
- client/server containers;
- packet capture сценарии;
- Grafana dashboard;
- benchmark scripts;
- failure scenarios: packet loss, latency, blocked UDP.

Почему это killer feature: рекрутер/инженер может быстро увидеть, что проект реальный.

## 9. Threat model docs

Документы:

- что защищаем;
- от кого не защищаем;
- какие fingerprints остаются;
- что делает padding;
- где возможны DNS leaks;
- чем отличается proxy от VPN.

Почему это killer feature: показывает зрелость мышления и безопасность без маркетинговых обещаний.

## 10. Rust quality bar

Обязательные вещи:

- typed config;
- `thiserror`/`anyhow` по месту;
- `tracing`;
- integration tests;
- property tests для packet framing;
- no panics in packet parser;
- fuzz target для protocol parser;
- CI: fmt, clippy, test.

Почему это killer feature: проект выглядит как production Rust, а не экспериментальный скрипт.

## Приоритет реализации

1. TUN + native QUIC tunnel.
2. Metrics + doctor.
3. Config schema + profiles.
4. External sing-box/Xray adapter.
5. Routing engine.
6. Obfuscation profiles.
7. Import/export profiles.
8. Demo lab.

## Источники

- Xray docs: https://xtls.github.io/
- sing-box docs: https://sing-box.sagernet.org/
- Hysteria2 protocol: https://v2.hysteria.network/docs/developers/Protocol/
- AmneziaWG: https://docs.amnezia.org/documentation/amnezia-wg/
- NaiveProxy: https://github.com/klzgrad/naiveproxy
