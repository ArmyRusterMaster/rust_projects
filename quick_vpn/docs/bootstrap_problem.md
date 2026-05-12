# Bootstrap problem: как подключаться, когда для VPN уже нужен VPN

Дата: 2026-05-12.

## Проблема

В странах с сильной фильтрацией пользователь часто не может:

- скачать клиент с официального сайта;
- открыть API сервиса;
- получить свежий список серверов;
- подключиться к известным IP/доменам VPN;
- пройти DPI, который узнает OpenVPN/WireGuard/QUIC/VLESS по признакам.

Поэтому "мощный VPN" должен решать не только data-plane, но и bootstrap-plane: как клиент получает первый рабочий маршрут, обновляет конфиг и выбирает transport.

## Главный вывод

Нужно проектировать `quick_vpn` как adaptive circumvention client:

1. В клиенте есть несколько независимых bootstrap channels.
2. Клиент пробует несколько transports параллельно.
3. Рабочие маршруты кешируются локально.
4. Параметры региона доставляются через signed tactics/config updates.
5. Серверные endpoints постоянно ротируются.
6. Есть режим "обычный HTTPS-сайт с секретным tunnel path".

## Подходы

### 1. Built-in seed endpoints

Клиент поставляется с небольшим набором seed endpoints.

Плюсы:

- просто;
- работает без внешней инфраструктуры;
- быстрый первый запуск.

Минусы:

- endpoints быстро банятся;
- обновление приложения становится критичным.

Как делать в quick_vpn:

- хранить только публичные bootstrap endpoints, не основные VPN-серверы;
- подписывать seed list;
- делать expiry;
- не полагаться только на этот метод.

### 2. Dynamic access keys

Идея как у Outline: пользователь получает ссылку, а сама ссылка указывает на удаленный JSON/конфиг, который можно менять без выдачи нового ключа.

Плюсы:

- можно менять серверы и порты без переустановки;
- удобно для маленьких групп пользователей;
- можно использовать несколько mirrors.

Минусы:

- URL конфига тоже может быть заблокирован;
- нужен trust model и подпись конфига.

Как делать в quick_vpn:

- `qvconf://` или обычный HTTPS URL;
- config payload подписан Ed25519;
- клиент принимает config только при валидной подписи;
- поддержать несколько mirrors и stale-cache fallback.

### 3. HTTPS camouflage / WebTunnel-like mode

Сервер выглядит как обычный HTTPS-сайт. Tunnel доступен только по секретному path/token, а обычный посетитель видит нормальный сайт.

Плюсы:

- похож на обычный web traffic;
- можно жить на `:443`;
- можно сосуществовать с реальным сайтом за reverse proxy;
- устойчивее к active probing.

Минусы:

- нужно реалистичное web-поведение;
- сложнее операционно;
- не решает блокировку домена целиком.

Как делать в quick_vpn:

- `webtunnel` transport поверх HTTPS/WebSocket-like stream;
- reverse proxy mode;
- secret path выдается через signed config;
- fallback сайт должен быть настоящим статическим сайтом или backend'ом.

### 4. Snowflake-like volunteer rendezvous

Пользователь подключается через короткоживущие volunteer proxies. Broker помогает найти рабочий proxy, а traffic выглядит похожим на WebRTC/video-call сценарии.

Плюсы:

- IP быстро меняются;
- сложнее заблокировать весь пул;
- хорошо для emergency access.

Минусы:

- сложно строить и модерировать;
- variable performance;
- нужны abuse controls;
- нужна инфраструктура broker/rendezvous.

Как делать в quick_vpn:

- не первым этапом;
- спроектировать `rendezvous` trait;
- позже сделать P2P relay mode отдельно от основного VPN.

### 5. Psiphon-style parallel dialing

Клиент одновременно пробует разные серверы, протоколы и параметры обфускации, выбирает первый успешный/лучший.

Плюсы:

- быстро находит рабочий маршрут;
- адаптируется к регионам;
- не требует от пользователя понимать протоколы.

Минусы:

- больше сетевого шума;
- сложнее телеметрия и rate limits.

Как делать в quick_vpn:

- `dialer` запускает N кандидатов с лимитом concurrency;
- кандидаты имеют score;
- успех кешируется;
- часть попыток всегда уходит на exploration;
- failures обновляют локальный score.

### 6. Tactics / region profiles

Сервер доставляет подписанные параметры для региона: какие transports пробовать, какие SNI/ALPN профили, какие ports, какие mirrors.

Плюсы:

- быстро реагирует на блокировки;
- можно тонко настраивать Россию/Иран/Китай/Туркменистан и т.д.;
- клиент остается универсальным.

Минусы:

- tactics endpoint сам должен быть доступен;
- плохая tactics-конфигурация может сломать регион.

Как делать в quick_vpn:

- signed tactics file;
- versioned schema;
- TTL;
- staged rollout;
- fallback на previous known-good tactics.

### 7. Out-of-band distribution

Конфиги и mirrors можно доставлять через каналы, которые обычно доступны дольше основного сайта:

- email;
- messenger bot;
- QR/share links;
- GitHub/GitLab releases;
- app store description/update metadata;
- static mirrors;
- friend-to-friend sharing;
- dynamic access key hosters.

Важно: клиент должен уметь принять config из текста/QR/file и проверить подпись.

## Что реализовать в quick_vpn

Минимальный набор:

- signed dynamic config;
- multi-mirror config fetch;
- local cache;
- parallel dialer;
- HTTPS/WebTunnel-like transport;
- external Xray/sing-box adapter как fallback;
- diagnostics: показать, какие bootstrap каналы работают.

Расширенный набор:

- region tactics;
- seed endpoint rotation;
- P2P/volunteer relay research mode;
- CDN/static mirror config distribution;
- import from QR/link.

## Источники

- Tor circumvention and pluggable transports: https://support.torproject.org/tor-browser/circumvention/unblocking-tor/
- Tor WebTunnel: https://blog.torproject.org/introducing-webtunnel-evading-censorship-by-hiding-in-plain-sight/
- Tor Snowflake: https://support.torproject.org/en-US/censorship/
- Psiphon tunnel core: https://github.com/Psiphon-Labs/psiphon-tunnel-core
- Outline Dynamic Access Keys: https://developer.getoutline.org/vpn/management/dynamic-access-keys
- Lantern adaptive routing: https://lantern.io/en/beta-circumvention
