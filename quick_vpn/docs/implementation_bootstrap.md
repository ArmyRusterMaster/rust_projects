# План реализации bootstrap/circumvention layer

Дата: 2026-05-12.

## Цель

Сделать так, чтобы `quick_vpn` мог подключаться в сетях, где:

- основной сайт заблокирован;
- часть серверов заблокирована;
- один или несколько протоколов не работают;
- пользователь не понимает, какой transport выбрать.

## Модули

### `config`

Ответственность:

- typed config;
- signed dynamic config;
- config bundles;
- expiry;
- validation;
- local cache.

API:

```text
load_local()
fetch_mirrors()
verify_signature()
merge_tactics()
save_last_good()
```

### `bootstrap`

Ответственность:

- seed endpoints;
- dynamic config URLs;
- mirrors;
- stale-cache policy;
- out-of-band import.

API:

```text
resolve_bootstrap_sources()
fetch_bootstrap_config()
rank_sources()
```

### `dialer`

Ответственность:

- построить candidate list;
- запускать parallel connection attempts;
- timeout/cancellation;
- score update;
- choose best session.

API:

```text
build_candidates(profile)
dial_parallel(candidates)
probe_health(session)
record_result(candidate, outcome)
```

### `transport`

Backend'ы:

- `quic_native`;
- `webtunnel_https`;
- `tcp_tls`;
- `external_xray`;
- `external_sing_box`;
- `future_p2p_relay`.

Trait:

```text
TransportBackend
  name()
  capabilities()
  dial(candidate)
  health()
  shutdown()
```

### `doctor`

Диагностика:

- can fetch config;
- DNS works;
- UDP reachable;
- TCP/443 reachable;
- TLS handshake works;
- QUIC handshake works;
- MTU estimate;
- TUN permission;
- external backend executable available.

Вывод должен быть понятным:

```text
UDP/QUIC: blocked or timeout
HTTPS/WebTunnel: reachable
Config mirror 1: blocked
Config mirror 2: ok
Suggested profile: webtunnel_https
```

## Config bundle

Пример структуры без реальных endpoints:

```toml
version = 1
region = "ru"
issued_at = "2026-05-12T00:00:00Z"
expires_at = "2026-05-19T00:00:00Z"

[[endpoints]]
id = "edge-a"
address = "example.invalid"
port = 443
public_key = "base64..."

[[transports]]
name = "webtunnel_https"
endpoint = "edge-a"
priority = 100

[[transports]]
name = "quic_native"
endpoint = "edge-a"
priority = 60

[[mirrors]]
url = "https://example.invalid/qvconf.json"
priority = 100
```

Подпись хранить отдельно или envelope-форматом:

```json
{
  "payload": "...canonical-json-or-cbor...",
  "signature": "...ed25519...",
  "key_id": "stable-key-id"
}
```

## Parallel dialing algorithm

1. Загрузить last-good config.
2. Параллельно начать fetch свежих mirrors.
3. Построить candidates из last-good + fresh config.
4. Отсортировать по score и regional tactics.
5. Запустить ограниченное число попыток.
6. Первый успешный transport дает usable session.
7. Еще 1-2 кандидата можно проверить фоном как backup.
8. Результаты записать в local score database.

## Security requirements

- Все remote configs должны быть подписаны.
- У config должен быть expiry.
- Нельзя логировать секретные links/tokens.
- External backend configs писать в приватную директорию.
- Secret path/token не должен попадать в process list.
- Fallback website не должен становиться open proxy.
- Нужны rate limits на handshake и probe paths.

## Что отложить

- Полный VLESS/REALITY с нуля.
- Volunteer P2P relay.
- Browser-grade TLS fingerprinting в чистом Rust.
- Автоматическое создание доменов/CDN.

## MVP acceptance criteria

- Клиент подключается через `quic_native` в обычной сети.
- Клиент подключается через `webtunnel_https`, если UDP выключен.
- Клиент может импортировать signed config из файла.
- Клиент может обновить config через mirror.
- Клиент кеширует last-good config.
- `doctor` показывает понятную причину failure.
- Есть integration test для выбора fallback transport.

## Источники

- Outline Dynamic Access Keys: https://developer.getoutline.org/vpn/management/dynamic-access-keys
- Psiphon Tunnel Core: https://github.com/Psiphon-Labs/psiphon-tunnel-core
- Tor WebTunnel: https://support.torproject.org/glossary/webtunnel/
- Tor Snowflake: https://support.torproject.org/en-US/censorship/
- Lantern adaptive routing: https://lantern.io/en/beta-circumvention
