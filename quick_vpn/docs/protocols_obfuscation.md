# Лучшие протоколы и способы обфускации

Дата: 2026-05-12.

## Рейтинг для quick_vpn

### 1. QUIC-native tunnel

Зачем: лучший первый transport для собственного Rust-проекта.

Плюсы:

- `quinn` дает production-grade QUIC API.
- DATAGRAM подходит для IP/UDP-like payload.
- Streams подходят для TCP/control-plane.
- Хорошая база для reconnect, migration, congestion metrics.

Минусы:

- UDP/QUIC может блокироваться целиком.
- Нужно аккуратно работать с MTU и datagram size.

Где применять: default transport для MVP.

### 2. VLESS + REALITY + XTLS Vision

Зачем: сильный compatibility target с Xray ecosystem.

Плюсы:

- VLESS легкий и stateless.
- REALITY маскирует TLS-handshake под target site behavior.
- XTLS Vision оптимизирует TLS-like flows и уменьшает лишнее копирование.
- Широкая поддержка клиентами.

Минусы:

- Полная совместимость сложна.
- Нужны точные TLS/fingerprint детали.
- Лучше начинать через external Xray/sing-box adapter.

Где применять: compatibility mode, не первый core protocol.

### 3. Hysteria2-like QUIC transport

Зачем: высокая скорость и устойчивость на плохих UDP-сетях.

Плюсы:

- TCP/UDP proxy поверх QUIC.
- HTTP/3 masquerade при невалидной auth.
- Salamander obfuscation.
- Bandwidth negotiation.
- Port hopping.

Минусы:

- Если сеть режет QUIC/HTTP3, нужен fallback.
- Salamander делает traffic random-looking, а не browser-looking.

Где применять: performance/unstable network profile.

### 4. TUIC-like transport

Зачем: компактный QUIC transport для TCP/UDP forwarding.

Плюсы:

- Проще VLESS/REALITY.
- Хорош для mobile/latency.
- Логично ложится на Rust QUIC stack.

Минусы:

- Меньше camouflage-фич.
- Меньше бизнес-ценности, чем VLESS compatibility.

Где применять: low-latency profile.

### 5. AmneziaWG-like obfuscated WireGuard mode

Зачем: референс для L3 VPN с сильной UDP packet obfuscation.

Плюсы:

- Сильная идея: оставить криптографию, менять transport signatures.
- Dynamic headers.
- Packet length randomization.
- Signature packets и junk-train.
- Хороший UX: WireGuard-like speed.

Минусы:

- Это отдельная protocol family.
- Не стоит копировать WireGuard crypto/protocol без нужды.

Где применять: изучить идеи obfs layer, но не делать первым.

### 6. Shadowsocks 2022 + plugins

Зачем: простота, совместимость, URI/import.

Плюсы:

- Легкий proxy.
- Понятная plugin model.
- Хорош для embedded.

Минусы:

- Старые plugin-based obfs слабее современных REALITY/Hysteria/AmneziaWG approaches.
- SIP003 plugins в основном TCP-only.

Где применять: import/export и simple proxy profile.

## Способы обфускации

### TLS/browser fingerprint mimicry

Идея: сделать TLS ClientHello и HTTP behavior похожими на реальные браузеры.

Полезно для:

- VLESS/REALITY ecosystem.
- NaiveProxy-like frontend mode.
- HTTPS camouflage.

Риски:

- Ручная имитация быстро устаревает.
- Лучше использовать проверенный backend или системную TLS-библиотеку с fingerprint support.

### REALITY-like handshake camouflage

Идея: невалидный клиент видит поведение target-сайта, валидный клиент получает tunnel.

Полезно для:

- Защиты от active probing.
- Скрытия факта proxy на открытом порту.

Риски:

- Неправильный target/fallback может создать abuse surface.
- Нужно строго ограничивать forwarded fallback behavior.

### HTTP/2/HTTP/3 masquerading

Идея: внешний вид обычного HTTPS/HTTP3 сервера, proxy включается только после auth.

Полезно для:

- Hysteria2-like behavior.
- NaiveProxy-like architecture.
- Corporate/network environments, где web traffic естественен.

Риски:

- Поведение HTTP-сервера должно быть реалистичным.
- Нужны нормальные certs, headers, status codes, timing.

### Packet padding

Идея: менять размер первых frames/packets и чувствительных control messages.

Полезно для:

- Снижения length-based fingerprinting.
- Скрытия handshake pattern.

Риски:

- Больше bandwidth.
- Неправильный padding сам становится fingerprint.

### Randomized keepalive and timing

Идея: избегать идеально периодических ping/keepalive.

Полезно для:

- Мобильных сетей.
- NAT keepalive.
- Уменьшения простых timing signatures.

Риски:

- Слишком агрессивная рандомизация ухудшает battery/latency.

### UDP packet masking

Идея: обернуть UDP/QUIC packets в дополнительный слой с salt/key/padding.

Полезно для:

- Сетей, которые режут узнаваемый QUIC.
- Hysteria Salamander-like режимов.

Риски:

- Random-looking traffic может быть отдельным признаком.
- Ломается совместимость с обычным QUIC/HTTP3.

### Port hopping

Идея: менять UDP-порт по расписанию или случайному интервалу.

Полезно для:

- Быстрой реакции на блокировку конкретного порта.
- Hysteria-like profiles.

Риски:

- Требует синхронизации клиента/сервера.
- Нужны firewall/NAT правила.

### Routing and split tunneling

Идея: не весь traffic должен идти через tunnel; правила по доменам, IP, процессам, приложениям.

Полезно для:

- UX.
- Скорости.
- Меньше подозрительного объема на один tunnel.

Риски:

- DNS leak.
- Сложность на Android/iOS.

## Практический выбор для quick_vpn

MVP:

- QUIC-native.
- TUN.
- Control stream + DATAGRAM.
- MTU handling.
- Metrics.

Next:

- Padding.
- Randomized keepalive.
- Port hopping.
- External Xray/sing-box adapter.

Advanced:

- VLESS-lite parser/framing.
- REALITY-compatible research prototype.
- Hysteria2-like mode.
- Rule-based routing + DNS/FakeIP.

## Источники

- RFC 9221 QUIC DATAGRAM: https://www.rfc-editor.org/rfc/rfc9221.html
- Hysteria2 protocol: https://v2.hysteria.network/docs/developers/Protocol/
- Hysteria2 client config: https://v2.hysteria.network/docs/advanced/Full-Client-Config/
- Xray VLESS: https://xtls.github.io/en/config/inbounds/vless.html
- Xray REALITY: https://xtls.github.io/en/config/transports/reality.html
- Xray transport fingerprint: https://xtls.github.io/en/config/transport.html
- sing-box Hysteria2: https://sing-box.sagernet.org/configuration/outbound/hysteria2/
- AmneziaWG: https://docs.amnezia.org/documentation/amnezia-wg/
- NaiveProxy: https://github.com/klzgrad/naiveproxy
- Shadowsocks SIP003: https://shadowsocks.org/doc/sip003.html
