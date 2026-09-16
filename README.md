# Oxide

[![CI](https://github.com/HrushikeshAnandSarangi/oxide/actions/workflows/ci.yml/badge.svg)](https://github.com/HrushikeshAnandSarangi/oxide/actions/workflows/ci.yml)

> Infrastructure as software — deterministic builds with Nix, isolated runtimes with Docker, proxied with Pingora.

Oxide is a self-hosted application deployment platform built entirely in Rust. It solves the version mismatch problem at the build layer using Nix for reproducible, deterministic builds, and isolates runtime concerns separately using Docker — two tools doing two distinct jobs rather than one tool doing both poorly.

The entire backend is written in Rust with Axum. Deployments target a single Ubuntu VM with a clean base image.

---

## Demo

> Demo coming soon.

---

## The Problem Oxide Solves

Most deployment platforms conflate two separate concerns:

- **Build reproducibility** — will this build produce the same artifact on any machine, any time?
- **Runtime isolation** — will this process stay contained and not affect other running services?

Tools like Docker are often used for both. This works but at the cost of large images, slow builds, and non-deterministic layer caching. Oxide separates these responsibilities explicitly — Nix owns the build, Docker owns the runtime. The tradeoff is a leaner, more predictable deployment pipeline where each tool does exactly one thing.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Incoming Traffic                   │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│              Pingora Proxy (Rust)                   │
│         Route · Load balance · TLS termination      │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│               Axum API Server (Rust)                │
│      Deploy · Status · Config · Health endpoints    │
└──────┬───────────────────────────┬──────────────────┘
       │                           │
┌──────▼──────┐           ┌────────▼────────┐
│  Nix Build  │           │  Docker Runtime │
│  Layer      │           │  Layer          │
│             │           │                 │
│ Deterministic│          │ Isolated process│
│ reproducible │          │ management      │
│ artifacts    │          │ per deployment  │
└─────────────┘           └─────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│              Ubuntu VM (Base Image)                 │
└─────────────────────────────────────────────────────┘
```



## Tech Stack

| Component | Technology | Reason |
|-----------|-----------|--------|
| API server | Axum (Rust) | Type-safe, async, minimal overhead |
| Proxy | Pingora (Rust) | Programmatic routing, Rust-native |
| Build system | Nix | Deterministic, reproducible artifacts |
| Runtime isolation | Docker | Process containment, resource limits |
| Metrics | Prometheus + Grafana | Live operational dashboards, scraped from `/metrics` |
| Telemetry ETL | Redis Streams + Postgres | Durable event log for historical deploy/build analytics |
| Base OS | Ubuntu (VM) | Clean, minimal, reproducible base |

---

## Features

### Working Now

- **Deterministic builds via Nix** — identical artifacts across any host, version mismatch eliminated at build time
- **Docker runtime isolation** — each deployed app runs in an isolated container, runtime concerns separated from build concerns
- **Axum API** — deploy, status, config, health, and metrics endpoints
- **Pingora proxy** — programmatic Rust-native traffic routing to deployed services
- **Health monitoring** — periodic per-deployment health checks with automatic route removal and status update on failure
- **Prometheus metrics + Grafana dashboards** — deploy counts by status, build duration, active containers, health-check failures; `docker compose up -d` brings up the whole stack locally
- **Telemetry ETL (Redis Streams)** — deployment lifecycle events are streamed via Redis and loaded into a Postgres event log by the `telemetry` consumer, for historical analytics independent of live metrics
- **Tests** — unit coverage for proxy routing/state, crypto round-trips, and Nix artifact resolution

### In Progress

- **Single VM end-to-end deployment** — full pipeline from source to running container on Ubuntu VM
- **CI/CD pipeline** — GitHub Actions for automated build and deployment
- **Linting and formatting** — `clippy` + `rustfmt` enforced
- **Benchmarks** — Pingora routing and Axum endpoint throughput
- **Integration tests** — full deploy() pipeline against a real Postgres/Docker/Nix environment

### Planned

- **Multi-app routing** — multiple applications on a single VM with path/subdomain routing via Pingora
- **Rollback** — one-command rollback to previous Nix-built artifact
- **TLS** — automatic certificate provisioning

---

## Getting Started

### Prerequisites

- Rust 1.75+
- Nix (with flakes enabled)
- Docker
- Ubuntu VM (local or cloud)

**Install Nix:**
```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | \
  sh -s -- install
```

Enable flakes in `~/.config/nix/nix.conf`:
```
experimental-features = nix-command flakes
```

### Clone and Build

```bash
git clone https://github.com/HrushikeshAnandSarangi/oxide
cd oxide
cargo build --release
```

### Run Locally

One-time setup (Postgres, Redis, Prometheus, Grafana via `docker compose`, migrations, build; requires the Docker daemon already running):

```bash
./scripts/dev_setup.sh
npm run setup
```

Then bring up the whole architecture — backing services, the `api` process (Axum API on `:3001` incl. `/metrics`, plus the Pingora proxy on `:8000`, spawned in-process), the `telemetry` ETL consumer, and the `web/` control panel (`:3000`), all at once:

```bash
npm run start
```

`web/` is Oxide's dashboard: create a project (with its Git repo URL), trigger a deploy, and watch deployment status — it proxies `/api/*` to the Rust API (see `web/next.config.ts`). Stop everything with `npm run stop` (stops the docker-compose services; `Ctrl+C` stops the Rust/Next processes).

Prefer running each piece by hand instead? `cargo run -p api`, `cargo run -p telemetry`, and `npm --prefix web run dev` individually, after `docker compose up -d`.

#### Port map

| Port | Service |
|---|---|
| 3000 | `web/` control panel (Next.js dev server) |
| 3001 | Oxide API (Axum) |
| 3002 | Grafana |
| 5432 | Postgres |
| 6379 | Redis |
| 8000 | Pingora proxy |
| 9090 | Prometheus |

### Run Tests

```bash
cargo test --workspace
```

### Benchmarks

```bash
cargo bench --workspace
```

See [benchmarks.md](benchmarks.md) for measured results (proxy routing, crypto, metrics encoding).

---
