# Benchmarks

All benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs) and live next to the code they measure:

- [`proxy/benches/routing_bench.rs`](proxy/benches/routing_bench.rs)
- [`common/benches/crypto_bench.rs`](common/benches/crypto_bench.rs)
- [`api/benches/api_bench.rs`](api/benches/api_bench.rs)

Run them yourself:

```bash
cargo bench --workspace
```

Results below were measured on a WSL2 Ubuntu VM (Windows 11 host), debug-symbol `bench` profile, September 2026. Criterion numbers are `[lower bound, estimate, upper bound]` from 100 samples; the table shows the point estimate. These are microbenchmarks of individual functions in isolation, not end-to-end request latency — CI only compiles the benchmark suites (`cargo check --workspace --benches`) to catch rot; it does not run them, since Criterion's statistical runs are too slow/noisy for a CI gate.

## Proxy routing (`resolve_target`)

The function Pingora's `upstream_peer` calls once per incoming request to map a `Host` header to a backend port ([proxy/src/router.rs](proxy/src/router.rs)), backed by a `DashMap` in [proxy/src/state.rs](proxy/src/state.rs).

| Registered routes | Time | Throughput |
|---|---|---|
| 1 | 265.65 ns | ~3.76M/s |
| 8 | 263.44 ns | ~3.80M/s |
| 64 | 268.63 ns | ~3.72M/s |
| 1,000 | 275.24 ns | ~3.63M/s |
| 10,000 | 277.74 ns | ~3.60M/s |
| miss (unregistered subdomain) | 201.57 ns | ~4.96M/s |

**Takeaway:** flat at ~270ns from 1 to 10,000 routes — the `DashMap` lookup is effectively O(1) and doesn't degrade as more apps are deployed concurrently onto the same proxy.

## Crypto (AES-256-GCM env var encryption)

Used for project environment variables ([common/src/crypto.rs](common/src/crypto.rs)); decrypted once per configured env var on every deploy ([controller/src/deployment_service.rs](controller/src/deployment_service.rs)).

| Operation | Value size | Time |
|---|---|---|
| Encrypt | short (`"production"`) | 1.6655 µs |
| Encrypt | long (Postgres connection string, ~120 chars) | 1.8905 µs |
| Decrypt | short | 670.34 ns |
| Decrypt | long | 851.86 ns |

**Takeaway:** sub-2µs even for a full connection-string-length secret — never a bottleneck in the deploy path, even with dozens of env vars per project.

## API metrics endpoint (`/metrics`)

What runs on every Prometheus scrape against the running `api` process ([controller/src/metrics.rs](controller/src/metrics.rs)).

| Operation | Time |
|---|---|
| `prometheus::gather()` + text-encode (4 registered metrics) | 10.523 µs |
| Increment a labeled counter (`oxide_deployments_total`) | 50.475 ns |
| Observe a histogram value (`oxide_build_duration_seconds`) | 18.846 ns |

**Takeaway:** at a typical 5-15s Prometheus scrape interval, ~10µs of encoding overhead is negligible; the per-event counter/histogram calls made inline in the deploy and health-check paths (tens of nanoseconds) don't meaningfully affect deploy latency.

## What these numbers do and don't tell you

These are hot-path microbenchmarks (routing lookup, crypto primitive, metrics encoding) run in isolation — they say the infrastructure itself isn't a bottleneck. They are **not** a measurement of end-to-end deploy time (dominated by `nix build` and Docker image build/start, both of which are I/O- and network-bound and vary per project) or of the Pingora proxy's actual request throughput under load (would need a real HTTP load-testing tool like `wrk` or `oha` against a running instance, not a Criterion microbenchmark). Those are natural next additions once there's a live deployment to point a load generator at.
