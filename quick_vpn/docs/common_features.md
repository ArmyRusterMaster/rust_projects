# Общие фичи аналогов

Дата: 2026-05-12.

Эти функции повторяются в сильных проектах: Xray, sing-box, Hysteria2, NaiveProxy, AmneziaWG, Outline, Shadowsocks ecosystem.

## Core networking

- Client/server mode.
- TCP forwarding.
- UDP forwarding.
- TUN mode для всего трафика устройства.
- SOCKS5/HTTP local proxy mode.
- Transparent proxy mode.
- IPv4 и IPv6.
- NAT table для UDP sessions.
- MTU management.
- Reconnect и session recovery.
- Connection migration или быстрый re-dial.

## Protocol layer

- Несколько transports.
- Отдельный control-plane и data-plane.
- QUIC streams для надежных flows.
- QUIC DATAGRAM или UDP-like channel для unreliable payload.
- Multiplexing.
- Per-user auth.
- Версионирование protocol messages.
- Fallback для unsupported/unauthorized clients.

## Obfuscation and anti-probing

- TLS/browser fingerprint selection.
- HTTP/2 или HTTP/3 masquerade.
- REALITY-like target fallback.
- Padding.
- Randomized keepalive.
- Packet size randomization.
- Dynamic headers.
- Junk packets перед handshake.
- Port hopping.
- Frontend/reverse proxy deployment.

## Routing

- Domain rules.
- Domain suffix/keyword/regex.
- IP CIDR rules.
- Geo/rule-set based routing.
- Port rules.
- Protocol sniffing.
- Process-based rules.
- Android package-based rules.
- Interface-based rules.
- Split tunneling.
- Direct/block/proxy actions.

## DNS

- Built-in DNS resolver.
- DoH/DoQ/DoT/HTTP3 DNS support.
- FakeIP.
- Hosts override.
- DNS leak prevention.
- Per-route DNS strategy.

## Operations

- Config validation.
- Dry-run mode.
- Health checks.
- Traffic stats.
- Prometheus metrics.
- Structured logs.
- Admin API.
- User quotas.
- Expiration dates.
- Import/export profiles.
- Backup/restore.

## UX

- One-command server install.
- Simple client profile link.
- QR code/profile export.
- Human-readable diagnostics.
- "Why connection failed" checks.
- Safe defaults.
- Minimal config for basic use.
- Advanced config for power users.

## Security

- Explicit threat model.
- No plaintext secrets in logs.
- Key rotation plan.
- Certificate pinning or controlled PKI.
- Replay protection where relevant.
- Rate limits.
- Abuse prevention for fallback/proxy behavior.
- Clear separation of auth, transport, routing and admin API.

## Testing

- Unit tests for protocol encoding.
- Integration tests with loopback TUN or mock transport.
- Packet-size regression tests.
- Load tests.
- Latency/throughput benchmarks.
- Compatibility tests against external backends where possible.
- CI with fmt, clippy, test.

## Источники

- Xray docs: https://xtls.github.io/
- sing-box TUN: https://sing-box.sagernet.org/configuration/inbound/tun/
- sing-box route: https://sing-box.sagernet.org/configuration/route/
- Hysteria2 protocol: https://v2.hysteria.network/docs/developers/Protocol/
- NaiveProxy README: https://github.com/klzgrad/naiveproxy/blob/master/README.md
- AmneziaWG: https://docs.amnezia.org/documentation/amnezia-wg/
- Shadowsocks SIP002/SIP003: https://shadowsocks.org/doc/sip002.html
