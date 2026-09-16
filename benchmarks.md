# Benchmarks

All benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs) and live next to the code they measure:

- [`proxy/benches/routing_bench.rs`](proxy/benches/routing_bench.rs)
- [`common/benches/crypto_bench.rs`](common/benches/crypto_bench.rs)
- [`api/benches/api_bench.rs`](api/benches/api_bench.rs)

Run them yourself:

```bash
cargo bench --workspace
```

The three Criterion sections below (Proxy routing, Crypto, API metrics endpoint) are regenerated automatically by [.github/workflows/benchmarks.yml](.github/workflows/benchmarks.yml) on every GitHub Release, via [scripts/update_benchmarks.mjs](scripts/update_benchmarks.mjs) reading Criterion's `target/criterion/**/base/estimates.json`, and committed straight back to `main` — the numbers you're reading were produced by CI, not hand-typed. That means they reflect a shared GitHub Actions runner, not a dedicated machine — treat absolute values as noisier than a local run, but the *shape* (flat routing latency across route-table sizes, sub-2µs crypto, etc.) is what matters and is stable. Container size (further down) is not part of that automation; see its own section for why. Regular CI (on every push/PR) only compiles the benchmark suites (`cargo check --workspace --benches`) to catch rot, since a full statistical run is too slow/noisy for that gate.

<!-- BENCH:updated:START -->
_Last updated: 2026-09-16T22:53:04.163Z (automated, via .github/workflows/benchmarks.yml)_
<!-- BENCH:updated:END -->

## Proxy routing (`resolve_target`)

The function Pingora's `upstream_peer` calls once per incoming request to map a `Host` header to a backend port ([proxy/src/router.rs](proxy/src/router.rs)), backed by a `DashMap` in [proxy/src/state.rs](proxy/src/state.rs).

<!-- BENCH:proxy_routing:START -->
| Registered routes | Time | Throughput |
|---|---|---|
| 1 | 265.65 ns | ~3.76M/s |
| 8 | 263.44 ns | ~3.80M/s |
| 64 | 268.63 ns | ~3.72M/s |
| 1,000 | 275.24 ns | ~3.63M/s |
| 10,000 | 277.74 ns | ~3.60M/s |
| miss (unregistered subdomain) | 201.57 ns | ~4.96M/s |
<!-- BENCH:proxy_routing:END -->

**Takeaway:** flat at ~270ns from 1 to 10,000 routes — the `DashMap` lookup is effectively O(1) and doesn't degrade as more apps are deployed concurrently onto the same proxy.

## Crypto (AES-256-GCM env var encryption)

Used for project environment variables ([common/src/crypto.rs](common/src/crypto.rs)); decrypted once per configured env var on every deploy ([controller/src/deployment_service.rs](controller/src/deployment_service.rs)).

<!-- BENCH:crypto:START -->
| Operation | Value size | Time |
|---|---|---|
| Encrypt | short (`"production"`) | 1.665 µs |
| Encrypt | long (Postgres connection string, ~120 chars) | 1.890 µs |
| Decrypt | short | 670.34 ns |
| Decrypt | long | 851.86 ns |
<!-- BENCH:crypto:END -->

**Takeaway:** sub-2µs even for a full connection-string-length secret — never a bottleneck in the deploy path, even with dozens of env vars per project.

## API metrics endpoint (`/metrics`)

What runs on every Prometheus scrape against the running `api` process ([controller/src/metrics.rs](controller/src/metrics.rs)).

<!-- BENCH:api_metrics:START -->
| Operation | Time |
|---|---|
| `prometheus::gather()` + text-encode (4 registered metrics) | 10.523 µs |
| Increment a labeled counter (`oxide_deployments_total`) | 50.48 ns |
| Observe a histogram value (`oxide_build_duration_seconds`) | 18.85 ns |
<!-- BENCH:api_metrics:END -->

**Takeaway:** at a typical 5-15s Prometheus scrape interval, ~10µs of encoding overhead is negligible; the per-event counter/histogram calls made inline in the deploy and health-check paths (tens of nanoseconds) don't meaningfully affect deploy latency.

## Container size: Nix vs. naive Docker

Sample app: [examples/minimal-rust-app](examples/minimal-rust-app) — a trivial std-only Rust HTTP server, no dependencies, ~20 lines. Two ways of turning it into a container image:

- **Nix-built**: `nix build` (using `pkgs.pkgsStatic.rustPlatform.buildRustPackage`) produces a fully static musl binary (`ELF ... static-pie linked`, 544KB, stripped), then `docker build -f Dockerfile.nix ./result` copies just that binary into `FROM scratch`. No shared libraries, no package manager, no shell.
- **Naive baseline**: `docker build -f Dockerfile.naive .` — `FROM rust:1-bookworm`, `cargo build --release`, run from the same image. This isn't a strawman; it's the standard first Dockerfile most people (and most tutorials) actually write, and it's what you get if you never revisit the Dockerfile after `cargo init`.

| Image | Size |
|---|---|
| `oxide-minimal-naive` (naive baseline) | 2.19 GB |
| `oxide-minimal-nix` (Nix static + `scratch`) | 814 kB |

**~2,690x smaller (99.96% reduction).** Both images were verified to actually run and serve traffic correctly (`docker run` + `curl` against each) before recording these numbers — a smaller image that doesn't work isn't a real comparison.

### How this was measured

```bash
cd examples/minimal-rust-app
nix build
docker build -f Dockerfile.nix -t oxide-minimal-nix ./result
docker build -f Dockerfile.naive -t oxide-minimal-naive .
docker images | grep oxide-minimal
```

Measured via `docker images`' `SIZE` column (the number everyone actually looks at) on WSL2 Ubuntu, September 2026. Reproducible from a clean `nix build` + two `docker build`s — see [examples/minimal-rust-app/README.md](examples/minimal-rust-app/README.md).

### The important caveat

This is **not** an automatic property of using Oxide. [runtime/src/image.rs](runtime/src/image.rs) does a literal `docker build` against whatever `Dockerfile` sits in the Nix build's output — Nix guarantees the *build* is reproducible, not that the resulting image is small. If a deployed repo's Nix flake emits a `Dockerfile` that does `FROM ubuntu` and copies in a build toolchain, the image will be just as large as the naive baseline above. The size win here comes from combining Nix's reproducible builds with deliberately minimal final-stage Dockerfiles (static binary → `scratch`), which Nix makes easy to do reliably (no "works on my machine" toolchain drift) but doesn't enforce by itself.

## What these numbers do and don't tell you

These are hot-path microbenchmarks (routing lookup, crypto primitive, metrics encoding) run in isolation — they say the infrastructure itself isn't a bottleneck. They are **not** a measurement of end-to-end deploy time (dominated by `nix build` and Docker image build/start, both of which are I/O- and network-bound and vary per project) or of the Pingora proxy's actual request throughput under load (would need a real HTTP load-testing tool like `wrk` or `oha` against a running instance, not a Criterion microbenchmark). Those are natural next additions once there's a live deployment to point a load generator at.
