# Oxide

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
| Base OS | Ubuntu (VM) | Clean, minimal, reproducible base |

---

## Features

### Working Now

- **Deterministic builds via Nix** — identical artifacts across any host, version mismatch eliminated at build time
- **Docker runtime isolation** — each deployed app runs in an isolated container, runtime concerns separated from build concerns
- **Axum API** — deploy, status, config, and health endpoints
- **Pingora proxy** — programmatic Rust-native traffic routing to deployed services
- **Tests** — core deployment pipeline covered

### In Progress

- **Single VM end-to-end deployment** — full pipeline from source to running container on Ubuntu VM
- **CI/CD pipeline** — GitHub Actions for automated build and deployment
- **Linting and formatting** — `clippy` + `rustfmt` enforced
- **Benchmarks** — Pingora routing and Axum endpoint throughput

### Planned

- **Multi-app routing** — multiple applications on a single VM with path/subdomain routing via Pingora
- **Health monitoring** — per-deployment health checks with automatic restart on failure
- **Rollback** — one-command rollback to previous Nix-built artifact
- **TLS** — automatic certificate provisioning
- **Metrics** — deployment and request telemetry

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

```bash
# Start the Axum API server
cargo run --bin oxide-api

# Start the Pingora proxy
cargo run --bin oxide-proxy
```

### Run Tests

```bash
cargo test
```

---
