# Способы первичного подключения в сильно фильтруемых сетях

Дата: 2026-05-12.

Документ описывает engineering-паттерны для устойчивого доступа. Это не набор готовых конфигов, а список механизмов, которые можно легально реализовать и тестировать в своей инфраструктуре.

## 1. Multi-channel config retrieval

Клиент должен уметь получать signed config из нескольких источников:

- встроенный seed list;
- HTTPS mirrors;
- dynamic access key URL;
- локальный файл;
- QR/share text;
- messenger-delivered text;
- ранее закешированный config.

Правило: config без подписи не применять.

## 2. Signed dynamic config

Формат:

```text
payload:
  version
  issued_at
  expires_at
  region
  endpoints[]
  transports[]
  mirrors[]
  tactics
signature:
  key_id
  ed25519_signature
```

Зачем:

- можно менять endpoints без переустановки;
- можно отзывать плохие endpoints;
- можно доставлять региональные transport preferences.

## 3. Parallel candidate dialer

Вместо "попробовать один сервер" клиент строит список кандидатов:

```text
candidate = endpoint + transport + port + tls_profile + obfs_profile
```

Затем:

- запускает 3-6 попыток параллельно;
- ограничивает общий timeout;
- выбирает первый успешный tunnel;
- продолжает health-check альтернатив;
- обновляет локальные scores.

Это похоже на подход Psiphon: разные серверы и обфускации пробуются конкурентно, успешные варианты запоминаются.

## 4. HTTPS/WebTunnel-like mode

Сценарий:

- домен открывает обычный HTTPS-сайт;
- secret path переключает соединение в tunnel mode;
- reverse proxy маршрутизирует обычный web и tunnel на разные backend'ы;
- без валидного token сервер отвечает как обычный сайт.

Что важно:

- normal-looking TLS;
- normal-looking HTTP status/errors;
- real website fallback;
- rate limits;
- secret path rotation;
- no tunnel-specific banners.

## 5. MASQUE-like mode

MASQUE использует HTTP/3 mechanisms вроде CONNECT-UDP/CONNECT-IP. Для `quick_vpn` это перспективный compatibility target:

- выглядит как современный HTTP/3 traffic;
- ближе к стандартам, чем custom tunnel;
- потенциально совместим с enterprise/proxy tooling.

Минусы:

- HTTP/3/QUIC может быть заблокирован;
- реализация сложнее простого QUIC-native tunnel.

## 6. External backend fallback

Если собственный transport не подключается, клиент может запускать внешний backend:

- Xray-core;
- sing-box;
- Tor pluggable transport binary;
- Shadowsocks-compatible backend.

`quick_vpn` в этом режиме:

- генерирует config;
- запускает process;
- проверяет health;
- подключает локальный SOCKS/HTTP/TUN route;
- показывает diagnostics.

Это дает быстрый путь к VLESS/REALITY/WebTunnel/Snowflake-like compatibility без переписывания всего.

## 7. Bridge distribution

Модель как у Tor bridges:

- часть endpoints не публикуется в общем списке;
- endpoints выдаются малыми партиями;
- новые endpoints добавляются до того, как старые умрут;
- клиент держит reserve list.

Для quick_vpn:

- private endpoint pools;
- per-user endpoint assignment;
- rolling rotation;
- abuse controls;
- signed endpoint leases.

## 8. Volunteer/P2P relay

Модель как Snowflake/Lantern Unbounded:

- censored client получает short-lived relay;
- relay находится через broker/rendezvous;
- relay не является постоянным VPN-сервером;
- трафик может быть WebRTC-like или обычным TLS-like.

Для MVP не делать, но заложить интерфейсы:

- `RendezvousProvider`;
- `RelayCandidate`;
- `RelaySession`;
- anti-abuse limits;
- relay reputation.

## 9. Offline and friend-to-friend bootstrap

Клиент должен уметь импортировать:

- config text;
- QR;
- file;
- NFC/share sheet;
- compressed signed bundle.

Это важно, когда сайт и API уже заблокированы, но пользователь может получить конфиг от знакомого.

## 10. Local survivability

После первого успеха клиент обязан сохранять:

- working endpoint;
- working transport;
- last-good tactics;
- backup mirrors;
- server public keys;
- expiry/rotation data.

Если свежий config недоступен, клиент может использовать stale config до ограниченного срока.

## Источники

- Psiphon Tunnel Core README: https://github.com/Psiphon-Labs/psiphon-tunnel-core
- Outline Dynamic Access Keys: https://developer.getoutline.org/vpn/management/dynamic-access-keys
- Tor Bridges and pluggable transports: https://support.torproject.org/little-t-tor/tor-pluggable-transports/
- Tor WebTunnel overview: https://support.torproject.org/glossary/webtunnel/
- Lantern Unbounded: https://github.com/getlantern/unbounded
