# Аналоги и конкуренты

Дата: 2026-05-12.

## Xray-core

Что это: модульная proxy-платформа, наследник V2Ray-экосистемы.

Сильные стороны:

- VLESS, VMess, Trojan, Shadowsocks, SOCKS/HTTP, WireGuard-like сценарии.
- REALITY, XTLS Vision, XHTTP, gRPC, WebSocket, TCP, QUIC transports.
- Сильный routing/DNS layer.
- Fallback behavior для невалидных подключений.
- Большая совместимость с клиентами и панелями.

Слабые стороны:

- Сложность конфигурации.
- Go codebase, много внутренних абстракций.
- Повторить совместимость с нуля дорого.

Что взять в quick_vpn:

- Модульность inbound/outbound.
- Routing rules по domain/IP/port/process/user.
- Fallback для probe traffic.
- Совместимость через external adapter.

## sing-box

Что это: универсальная proxy-платформа с TUN, routing, DNS и множеством протоколов.

Сильные стороны:

- TUN inbound.
- VLESS, Shadowsocks, Trojan, Hysteria2, TUIC, WireGuard, Tor/SSH-like outbounds.
- Гибкие route rules: domain, CIDR, process, package name, network type, Wi-Fi SSID, interface.
- DNS stack: DoH, DoQ, HTTP3, FakeIP.
- Linux/Android/macOS/Windows сценарии.

Слабые стороны:

- Тоже Go и большая конфигурационная поверхность.
- Быстро меняющиеся поля конфига.

Что взять в quick_vpn:

- TUN-first design.
- `auto_detect_interface`/loop prevention.
- DNS/FakeIP как отдельный слой.
- Rule-set based routing.
- Android package/process-aware routing как будущую фичу.

## Hysteria2

Что это: TCP/UDP proxy поверх QUIC с упором на скорость и устойчивость в плохих сетях.

Сильные стороны:

- QUIC transport + unreliable datagrams.
- Маскировка под HTTP/3 при неавторизованном доступе.
- Salamander obfuscation для UDP/QUIC packet masking.
- Bandwidth negotiation и congestion-control настройки.
- Port hopping.

Слабые стороны:

- UDP/QUIC может блокироваться целиком.
- Salamander ломает совместимость со стандартным HTTP/3 behavior.

Что взять в quick_vpn:

- QUIC DATAGRAM для data-plane.
- HTTP/3-like camouflage как отдельный режим.
- Port hopping abstraction.
- Явные bandwidth profiles.

## TUIC

Что это: proxy поверх QUIC, часто используется для TCP/UDP tunneling с низкой задержкой.

Сильные стороны:

- QUIC-native design.
- Хорош для мобильных и нестабильных сетей.
- Поддержка TCP/UDP tunneling.
- Обычно проще, чем VLESS/REALITY ecosystem.

Слабые стороны:

- Меньше ecosystem-фич, чем у Xray/sing-box.
- UDP-зависимость.

Что взять в quick_vpn:

- QUIC-first transport.
- Простую модель TCP/UDP sessions.
- Connection migration/reconnect как важную часть UX.

## AmneziaWG

Что это: WireGuard-derived VPN с обфускацией транспортных признаков.

Сильные стороны:

- Сохраняет производительность и криптографическое ядро WireGuard.
- Меняет узнаваемые headers.
- Рандомизирует размеры handshake/data packets.
- Может имитировать распространенные UDP-протоколы.
- Обратная совместимость с WireGuard при выключенной маскировке.

Слабые стороны:

- Это отдельная WireGuard-линия, а не VLESS/QUIC proxy.
- Реализация на уровне WireGuard semantics сложна для быстрого Rust MVP.

Что взять в quick_vpn:

- Randomized packet sizes.
- Dynamic headers/ranges как идея для собственного UDP obfs layer.
- Presets для masking profiles.
- Четкое разделение криптографии и обфускации.

## NaiveProxy

Что это: proxy, использующий Chromium network stack для похожести на браузерный HTTPS-трафик.

Сильные стороны:

- Реальный Chrome/Chromium TLS/HTTP fingerprint.
- HTTP/2/HTTP/3 CONNECT-style tunneling.
- Application fronting через обычный frontend server.
- Padding первых frames.
- Хорошая защита от TLS parameter fingerprinting.

Слабые стороны:

- Chromium stack тяжелый.
- Сложно повторить в чистом Rust.

Что взять в quick_vpn:

- Не имитировать браузер вручную, если можно интегрироваться с проверенным backend.
- Padding первых frames.
- Frontend/reverse-proxy deployment mode.

## Shadowsocks + plugins

Что это: легкий encrypted proxy с plugin model.

Сильные стороны:

- Простая архитектура.
- SIP002 URI и SIP003 plugin model.
- Плагины: obfs, v2ray-plugin, Cloak, Kcptun и другие.
- Хорош для embedded/router use cases.

Слабые стороны:

- Старые obfs-подходы часто детектируются.
- SIP003 plugins в основном TCP-only.

Что взять в quick_vpn:

- Plugin process model.
- URI import/export.
- Простые профили для пользователей.

## Outline

Что это: user-friendly self-hosted VPN/proxy на базе Shadowsocks.

Сильные стороны:

- Очень простой setup.
- Управление ключами и пользователями.
- Open-source, self-hosted.
- Хороший UX для неадминов.

Слабые стороны:

- Не самый сильный anti-DPI стек.
- Меньше advanced routing/protocol features.

Что взять в quick_vpn:

- "Работает за 5 минут" как product goal.
- Простое создание пользователей.
- Client config links.
- Health/status UI или CLI.

## Панели: 3X-UI, Marzban, Marzneshin

Что это: панели управления поверх Xray/sing-box.

Сильные стороны:

- Multi-user management.
- Traffic limits, expiration, IP limits.
- Protocol templates.
- Import/export database.
- Server status monitoring.

Слабые стороны:

- Обычно не являются transport innovation.
- Без core backend малоценны.

Что взять в quick_vpn:

- User/account model.
- Quotas.
- Expiration.
- Admin API.
- Config templates.

## Источники

- Xray docs: https://xtls.github.io/
- sing-box docs: https://sing-box.sagernet.org/
- Hysteria2 protocol: https://v2.hysteria.network/docs/developers/Protocol/
- Hysteria2 config: https://v2.hysteria.network/docs/advanced/Full-Server-Config/
- AmneziaWG: https://docs.amnezia.org/documentation/amnezia-wg/
- NaiveProxy README: https://github.com/klzgrad/naiveproxy/blob/master/README.md
- Shadowsocks SIP003: https://shadowsocks.org/doc/sip003.html
- Shadowsocks SIP002: https://shadowsocks.org/doc/sip002.html
- 3X-UI overview: https://nstool.org/sites/3x-ui.html
