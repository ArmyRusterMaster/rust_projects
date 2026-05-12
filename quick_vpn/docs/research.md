# quick_vpn research notes

Дата: 2026-05-12.

## Цель MVP

Сделать простой userspace VPN поверх QUIC:

1. Клиент поднимает TUN-интерфейс.
2. IP-пакеты из TUN отправляются на сервер через QUIC DATAGRAM.
3. Сервер принимает IP-пакеты, маршрутизирует/форвардит их и возвращает ответы клиенту.
4. Надежный QUIC stream используется отдельно для control-plane: handshake приложения, версии протокола, keepalive, адреса, маршруты, лимиты MTU.

## Рекомендованный стек

- `quinn` как основной QUIC transport. Это pure-Rust, async API поверх Tokio, с rustls и поддержкой application datagrams. Для своего VPN-протокола это самый прямой путь: меньше низкоуровневого кода, чем с `quiche`.
- `tun-rs` для TUN/TAP. У него есть async-интеграция, Linux offload, multi-queue и мобильные сценарии через fd для Android/iOS.
- `tokio` как runtime.
- `bytes` для передачи datagram payload без лишних копий.
- `tracing` + `tracing-subscriber` для диагностики.
- `clap` для CLI.
- `rustls`/`rcgen` для TLS-конфигурации на раннем этапе; для реального использования лучше перейти на нормальную PKI или pinning.

## Альтернативы

- `s2n-quic`: сильная production-библиотека с большим набором провайдеров, тестированием, pacing, PMTU discovery, GSO и rustls/s2n-tls. Хороший вариант, если важнее зрелая инфраструктура и AWS-экосистема.
- `quiche`: низкоуровневая QUIC/HTTP/3 библиотека от Cloudflare. Дает больше контроля, но приложение само ведет socket I/O, event loop и timers. Для первого MVP это лишняя сложность.
- `tun`: нормальный вариант для Linux-only/простого TUN, но `tun-rs` выглядит лучше как базовый выбор из-за платформ, async API и performance features.
- `tun-tap`: слишком базовый и с заметными ограничениями; не брать для нового проекта.

## Протокольные решения

- Для своего протокола поверх QUIC использовать raw QUIC DATAGRAM, а не HTTP/3, если цель - быстрый собственный VPN.
- Если нужна совместимость с MASQUE/HTTP-прокси, ориентироваться на CONNECT-IP из RFC 9484: IP payloads идут через HTTP Datagrams, control-plane - через Capsule Protocol.
- QUIC DATAGRAM не гарантирует доставку и порядок. Это нормально для IP-пакетов: TCP внутри туннеля сам восстановится, UDP может теряться как в обычной сети.
- Всегда проверять negotiated datagram support и `max_datagram_size`; не отправлять пакет больше лимита.
- MTU туннеля лучше начать с 1200-1280 байт, затем добавить PMTU/fragmentation policy. QUIC DATAGRAM не фрагментирует payload на уровне QUIC.
- Для IPv4/IPv6 декрементировать TTL/Hop Limit при форвардинге, чтобы не создавать бесконечные routing loops.
- Не форвардить link-local трафик за пределы соответствующего туннельного интерфейса.

## Минимальная архитектура

- `src/main.rs`: CLI entrypoint.
- `src/client.rs`: TUN read/write loop, QUIC connect, reconnect policy.
- `src/server.rs`: QUIC accept loop, per-client session, packet forwarding.
- `src/proto.rs`: версия протокола, control messages, constants, error codes.
- `src/tun.rs`: thin wrapper над `tun-rs`.
- `src/route.rs`: настройка routes/platform-specific logic.

## Риски

- TUN создание требует root/CAP_NET_ADMIN на Linux и Android VpnService на Android.
- QUIC поверх UDP может блокироваться в некоторых сетях; позже можно добавить fallback через HTTP/3/MASQUE.
- UDP buffer sizes сильно влияют на throughput и latency.
- Надо явно проектировать auth: self-signed без pinning годится только для разработки.
- На мобильных платформах управление route/DNS отличается от Linux и должно быть отдельным слоем.

## Источники

- Quinn docs: https://docs.rs/crate/quinn/latest
- Quinn Connection datagrams API: https://docs.rs/quinn/latest/quinn/struct.Connection.html
- tun-rs docs: https://docs.rs/tun-rs
- s2n-quic docs: https://docs.rs/s2n-quic
- quiche docs: https://docs.quic.tech/quiche/
- RFC 9221, QUIC DATAGRAM: https://www.rfc-editor.org/rfc/rfc9221.html
- RFC 9297, HTTP Datagrams and Capsule Protocol: https://www.rfc-editor.org/rfc/rfc9297.html
- RFC 9484, Proxying IP in HTTP / CONNECT-IP: https://datatracker.ietf.org/doc/rfc9484/
