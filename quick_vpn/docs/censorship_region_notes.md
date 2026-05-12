# Региональные заметки: сильная фильтрация и Россия

Дата: 2026-05-12.

## Важное ограничение

Фильтрация меняется быстро. Документ описывает не "рабочие адреса", а классы проблем и архитектурные ответы. Для реального продукта нужен telemetry/diagnostics layer и регулярное обновление tactics.

## Типовые блокировки

### Endpoint blocking

Блокируются IP/домены известных VPN/proxy.

Ответ:

- endpoint rotation;
- private bridge pools;
- dynamic access keys;
- per-user endpoint assignment;
- WebTunnel-like co-location with ordinary HTTPS website.

### Protocol fingerprinting

DPI узнает OpenVPN, WireGuard, QUIC, Shadowsocks, VLESS/TLS по сетевым признакам.

Ответ:

- transport diversity;
- TLS/browser-like profiles;
- HTTPS/WebSocket-like tunnel;
- padding;
- randomized keepalive;
- fallback на external Xray/sing-box.

### API/config blocking

Даже если VPN-сервер жив, клиент не может получить список серверов.

Ответ:

- signed config mirrors;
- dynamic access key;
- embedded seeds;
- stale-cache mode;
- out-of-band import.

### Store/site blocking

Пользователь не может скачать приложение или открыть сайт.

Ответ:

- GitHub/GitLab releases;
- package repository mirrors;
- app store alternatives;
- checksum/signature verification;
- friend-to-friend distribution;
- minimal offline installer.

### UDP degradation/blocking

QUIC/WireGuard/Hysteria/TUIC могут перестать работать, если UDP режется или деградирует.

Ответ:

- TCP/TLS fallback;
- HTTPS/WebTunnel-like mode;
- SOCKS/HTTP local proxy mode;
- adaptive transport selection.

## Россия: что учитывать

Для России как для одного из целевых профилей стоит предполагать:

- популярные VPN-протоколы могут блокироваться точечно;
- UDP может работать нестабильно на отдельных провайдерах;
- известные домены и IP сервисов быстро попадают в списки;
- пользователю часто нужен простой "connect" без ручной настройки;
- Telegram/социальные/зеркальные каналы иногда становятся каналами доставки конфигов, но на них нельзя полагаться как на единственный bootstrap.

Рекомендуемый региональный профиль:

```text
ru-balanced:
  first_try:
    - webtunnel_https
    - external_vless_reality
    - quic_native_padded
  fallback:
    - tcp_tls
    - shadowsocks_2022
  bootstrap:
    - dynamic_config_mirrors
    - cached_last_good
    - imported_signed_bundle
  diagnostics:
    - udp_reachability
    - tls_sni_check
    - config_mirror_check
    - mtu_probe
```

Это не готовая схема подключения. Это профиль приоритетов для реализации и тестирования.

## Что нельзя делать как единственную стратегию

- Только WireGuard/OpenVPN.
- Только один домен API.
- Только один QUIC transport.
- Только публичный список серверов.
- Только ручные конфиги.
- Только "случайный padding" без измерений.

## Что стоит реализовать первым

1. `quick-vpn doctor --region ru`
2. Signed dynamic config.
3. Parallel dialing.
4. WebTunnel-like HTTPS mode.
5. External Xray/sing-box adapter.
6. Last-known-good cache.

## Метрики региона

Собирать без приватных URL/доменных данных:

- success/failure by transport family;
- handshake timeout buckets;
- UDP reachability;
- median RTT;
- config mirror availability;
- endpoint churn;
- disconnect reason.

Приватность важна: не логировать посещенные сайты, DNS-запросы пользователя и полный IP без необходимости.

## Источники

- Tor censorship circumvention manual: https://tb-manual.torproject.org/circumvention/
- Tor WebTunnel: https://blog.torproject.org/introducing-webtunnel-evading-censorship-by-hiding-in-plain-sight/
- Psiphon guide: https://www.psiphon.ca/fa_AF/psiphon-guide.html
- Psiphon tunnel core: https://pkg.go.dev/go.psiphon.dev/tunnel-core
- Lantern advanced circumvention: https://lantern.io/en/beta-circumvention
