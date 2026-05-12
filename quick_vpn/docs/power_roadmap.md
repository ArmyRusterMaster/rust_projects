# Power roadmap: как сделать quick_vpn максимально сильным проектом

Дата: 2026-05-12.

## Главный вывод

`quick_vpn` должен выглядеть не как "еще один VPN", а как production-grade networking platform:

- надежный VPN/tunnel core;
- adaptive transport selection;
- сильная диагностика;
- observability;
- опциональные Linux acceleration paths;
- честный threat model;
- воспроизводимый demo lab;
- тесты, benchmarks и документация уровня инженерного продукта.

`io_uring` и eBPF стоит добавить, но только как опциональные режимы запуска. Базовый путь должен оставаться простым: Tokio + quinn + tun-rs.

## Что еще изучить

### Linux networking

- TUN/TAP internals.
- Policy routing.
- nftables.
- conntrack.
- MTU/PMTU.
- UDP socket buffers.
- GRO/GSO.
- SO_REUSEPORT.
- Network namespaces для тестов.

### QUIC глубже

- QUIC DATAGRAM limits.
- stream vs datagram tradeoffs.
- congestion control.
- packet loss behavior.
- connection migration.
- keepalive/NAT rebinding.
- ALPN/TLS fingerprints.
- HTTP/3/MASQUE basics.

### Production security

- certificate pinning;
- signed configs;
- key rotation;
- replay protection;
- rate limiting;
- abuse prevention;
- secret handling;
- secure logging.

### Censorship/circumvention engineering

- Tor pluggable transports;
- WebTunnel;
- Snowflake;
- Psiphon tactics/parallel dialing;
- Outline dynamic access keys;
- sing-box/Xray routing and DNS models;
- failure telemetry without user traffic logging.

### Performance engineering

- flamegraphs;
- `perf`;
- eBPF profiling;
- Tokio task instrumentation;
- allocation profiling;
- lock contention;
- packet batching;
- zero-copy limits;
- realistic benchmarks under loss/latency.

## Что обязательно сделать в проекте

### 1. `quick-vpn doctor`

Самая сильная UX/ops-фича.

Проверяет:

- TUN permissions;
- UDP reachability;
- QUIC handshake;
- TCP/443 fallback;
- DNS leaks;
- route loops;
- MTU;
- config signature;
- external backend health;
- kernel feature support: `io_uring`, eBPF, TUN, BTF.

### 2. Demo lab

В репозитории должен быть lab, который можно запустить локально:

- client/server containers;
- network namespaces;
- packet loss/latency simulation;
- blocked UDP scenario;
- blocked config mirror scenario;
- Grafana/Prometheus;
- benchmark scripts.

### 3. Benchmarks

Метрики:

- throughput TCP;
- throughput UDP;
- p50/p95/p99 latency;
- reconnect time;
- CPU per Gbit/s;
- memory per session;
- log throughput;
- packet drops.

Сравнения:

- native QUIC;
- webtunnel HTTPS fallback;
- external Xray/sing-box mode;
- no obfs vs padded mode;
- normal logging vs io_uring logging.

### 4. Threat model

Отдельный документ:

- от кого защищаем;
- от кого не защищаем;
- какие fingerprints остаются;
- что делает padding;
- что делает WebTunnel-like fallback;
- какие риски у external backends;
- как не стать open proxy.

### 5. Compatibility layer

Импорт/экспорт:

- `vless://`;
- `ss://`;
- `hy2://`;
- `tuic://`;
- quick_vpn TOML;
- sing-box JSON subset;
- Xray JSON subset.

## io_uring для логов и диска

### Идея

Сделать отдельный high-throughput log writer:

- app пишет structured events в bounded channel;
- writer batch'ит записи;
- пишет append-only сегменты;
- делает rotation;
- опционально использует `io_uring`;
- опционально использует `O_DIRECT` только для специальных режимов.

### Где это реально полезно

- packet/event audit logs;
- high-volume metrics spool;
- offline debug traces;
- relay/server с тысячами сессий;
- сценарии, где нельзя терять события при backpressure.

### Где это может быть лишним

Для обычных application logs `tracing` + buffered writer часто достаточно. `io_uring` не должен быть первым делом, потому что:

- усложняет runtime model;
- Linux-only;
- требует аккуратного shutdown/flush;
- direct I/O имеет жесткие alignment requirements;
- неправильный `O_DIRECT` может быть медленнее обычного page cache.

### Рекомендация

Сделать 3 backend'а:

```text
log_writer = "buffered"
log_writer = "tokio_blocking"
log_writer = "io_uring"
```

`buffered` - default.

`io_uring` - feature flag:

```text
--features io-uring-logs
```

### Архитектура

```text
tracing layer
  -> bounded mpsc
  -> log batcher
  -> segment writer
  -> fsync policy
  -> rotation
```

Fsync policies:

- `never`: fastest, debug only;
- `interval_ms`;
- `every_batch`;
- `on_shutdown`;
- `critical_only`.

### Crates/options

- `io-uring`: низкоуровневый Rust binding.
- `glommio`: thread-per-core runtime на io_uring, Linux-only.
- `monoio`: thread-per-core runtime с io_uring/epoll/kqueue.
- `tokio-uring`: ближе к Tokio ecosystem, но отдельный runtime.

Практический выбор: начать с обычного `tracing` writer, затем отдельный `io_uring_log_writer` модуль на feature flag. Не переводить весь проект с Tokio на другой runtime до появления измеримой причины.

### Важные детали

- Использовать bounded queues, иначе логирование может съесть память.
- При переполнении иметь policy: drop debug, sample, block, emergency file.
- Никогда не писать secrets/tokens.
- Для `O_DIRECT` нужны aligned buffers и aligned offsets.
- Лучше сначала batch + page cache, потом сравнить с `O_DIRECT`.
- Обязательно benchmark на реальном устройстве, особенно Android/Termux/Linux VPS.

## eBPF как опция запуска

### Идея

Добавить Linux-only eBPF helpers через `aya`:

- observability;
- fast packet classification;
- DNS leak detection;
- route loop detection;
- per-flow counters;
- optional XDP/tc filters.

### Где eBPF полезен

#### Observability

- считать packets/bytes per interface;
- видеть drops;
- измерять latency на socket/flow level;
- определять, какой process генерирует traffic;
- отслеживать DNS leaks.

#### Policy/routing assist

- маркировать пакеты;
- early drop known-bad traffic;
- fast path для некоторых route decisions;
- защита от loopback tunnel loops.

#### Performance experiments

- XDP drop/pass counters;
- tc egress/ingress classifiers;
- per-CPU maps для counters.

### Где eBPF опасен/лишний

- требует root/CAP_BPF/CAP_NET_ADMIN;
- зависит от kernel version и BTF;
- eBPF verifier сложен;
- сложная отладка;
- плохая программа может сломать networking;
- на Android часто ограничено.

### Рекомендация

Сделать eBPF строго опциональным:

```text
quick-vpn run --ebpf=off
quick-vpn run --ebpf=observe
quick-vpn run --ebpf=policy
quick-vpn run --ebpf=xdp-experimental
```

Приоритет:

1. `observe`: counters, drops, DNS leak detection.
2. `policy`: packet marking / route-loop guard.
3. `xdp-experimental`: только для lab/bench.

### Crates/options

- `aya`: userspace eBPF loader and management.
- `aya-ebpf`: kernel-space eBPF programs in Rust.

Почему `aya`: Rust-native, без libbpf/bcc, поддерживает BTF и подходит под single-binary deployment model.

### Safety rules

- eBPF включается только явным флагом.
- `doctor` проверяет kernel/BTF/caps до запуска.
- Всегда есть fallback без eBPF.
- Maps имеют лимиты.
- Все packet parsing paths проверяют bounds до чтения.
- Встроить `unload`/cleanup.
- В lab добавить тест, что eBPF mode не ломает маршрутизацию.

## Killer additions сверх базового VPN

### Adaptive transport scoring

Клиент учится, какие transports работают в текущей сети:

- success rate;
- handshake time;
- disconnect rate;
- packet loss;
- region tactics;
- battery/network type.

### Failure-aware UX

Не просто "connection failed", а:

```text
UDP недоступен, пробую HTTPS fallback.
Config mirror 1 недоступен, mirror 2 работает.
TUN не поднят: нет CAP_NET_ADMIN.
MTU слишком большой: suggested 1280.
```

### Privacy-preserving telemetry

Собирать только технические агрегаты:

- transport success/failure;
- latency buckets;
- feature availability;
- app version;
- no visited domains;
- no raw user IP unless explicit debug mode.

### Replayable packet tests

Сохранить synthetic traces и прогонять:

- packet parser;
- routing engine;
- NAT table;
- reconnect;
- MTU edge cases;
- malformed frames.

### Fuzzing

Fuzz targets:

- config parser;
- protocol frame parser;
- VLESS-lite parser;
- URI import parser;
- route rule parser.

## Приоритетный план

1. `doctor`.
2. Typed config + signed dynamic config.
3. Native QUIC tunnel.
4. TUN routing + DNS leak guard.
5. Metrics/Prometheus.
6. Demo lab.
7. Parallel dialer + fallback transports.
8. External Xray/sing-box adapter.
9. Benchmarks.
10. `io_uring` log writer feature.
11. eBPF observe mode.
12. eBPF policy mode.
13. VLESS-lite compatibility research.

## Источники

- `io-uring` crate: https://docs.rs/crate/io-uring/latest
- Glommio: https://glommio.github.io/
- Glommio docs: https://docs.rs/glommio/latest/glommio/
- Monoio docs: https://docs.rs/monoio/latest/monoio/
- Tokio-uring: https://lib.rs/crates/tokio-uring
- Aya docs: https://docs.rs/aya/latest/aya/
- Aya eBPF docs: https://docs.rs/aya-ebpf
- Aya GitHub: https://github.com/aya-rs/aya
- Direct I/O notes: https://kernel-internals.org/io/direct-io/
