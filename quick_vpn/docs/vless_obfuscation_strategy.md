# Стратегия: мощный VPN/tunnel с VLESS и обфускацией

Дата: 2026-05-12.

## Короткий вывод

Идея сильная для портфолио, но правильнее строить не "клон Xray на Rust", а модульную VPN/tunnel-платформу:

1. Rust core: TUN, routing, sessions, auth, metrics, config validation.
2. Transport abstraction: QUIC-native, TCP/TLS-like, external Xray/sing-box adapter.
3. Obfuscation layer: padding, randomized timings, TLS/HTTP-like camouflage, optional UDP packet masking.
4. Совместимость: импорт/экспорт профилей, запуск внешних backend'ов, health checks.
5. Только потом частичная реализация VLESS-compatible transport.

## Почему не начинать с полного VLESS/REALITY

VLESS сам по себе относительно небольшой stateless-протокол с UUID-auth, но связка `VLESS + REALITY + XTLS Vision + XHTTP` живет внутри большой экосистемы Xray. Там важны детали TLS ClientHello, custom certificate verification, fallback на target, flow-control, XUDP, multiplexing и routing.

Ошибка в мелкой детали даст либо нестабильность, либо узнаваемый трафик. Для портфолио лучше показать надежный core и понятную архитектуру, чем недоделанную копию Xray.

## Рекомендуемый путь реализации

### Этап 1: собственный VPN core

- Linux TUN через `tun-rs`.
- Async networking через `tokio`.
- QUIC transport через `quinn`.
- Control stream: версия протокола, параметры сессии, MTU, keepalive, routes.
- Data plane: QUIC DATAGRAM для IP/UDP-like payload.
- Metrics: active sessions, bytes in/out, reconnect count, datagram drops, RTT.
- Config: `toml`/`yaml`, schema validation, dry-run.

### Этап 2: transport abstraction

Интерфейс:

```text
Transport
  connect(config) -> Session
  send_datagram(bytes)
  open_stream(target)
  recv()
  stats()
```

Backend'ы:

- `quic-native`: свой быстрый transport.
- `tcp-tls`: простой baseline для сравнения.
- `external-xray`: генерирует config, запускает Xray/sing-box, мониторит health.
- `external-sing-box`: TUN/routing/protocol backend через sing-box.
- `vless-lite`: будущий частичный VLESS-compatible режим.

### Этап 3: обфускация

Начать с безопасных и измеримых механизмов:

- padding для первых пакетов и control frames;
- randomized keepalive;
- traffic shaping presets;
- маскировка UDP packet sizes;
- fallback behavior для неавторизованных подключений;
- profile-based TLS fingerprint selection через внешний backend.

Не обещать "невидимость". В документации писать честно: цель - уменьшить устойчивые признаки и дать измеримые tradeoffs.

### Этап 4: совместимость с экосистемой

- Импорт URI: `vless://`, `ss://`, `hy2://`, `tuic://` как config-профили.
- Экспорт профилей для клиентов.
- Интеграция с Xray/sing-box как backend mode.
- Диагностика: проверка DNS, SNI, cert, UDP reachability, MTU.

## Чем это хорошо для работы

Проект показывает редкую комбинацию:

- Rust async/networking;
- userspace VPN/TUN;
- QUIC;
- protocol design;
- observability;
- security tradeoffs;
- production-style CLI/daemon;
- интеграция с существующими open-source системами.

## Ограничения и этика

Не хранить в репозитории готовые схемы для злоупотребления чужой инфраструктурой. Не обещать обход всех блокировок. Документацию держать в формате engineering research: протоколы, tradeoffs, тестирование, безопасность, совместимость.

## Источники

- Xray VLESS: https://xtls.github.io/en/config/inbounds/vless.html
- Xray VLESS protocol: https://xtls.github.io/en/development/protocols/vless.html
- Xray REALITY: https://xtls.github.io/en/config/transports/reality.html
- Xray transport/uTLS fingerprint: https://xtls.github.io/en/config/transport.html
- sing-box TUN: https://sing-box.sagernet.org/configuration/inbound/tun/
- sing-box route rules: https://sing-box.sagernet.org/configuration/route/rule/
- Hysteria2 protocol: https://v2.hysteria.network/docs/developers/Protocol/
- AmneziaWG: https://docs.amnezia.org/documentation/amnezia-wg/
