#!/bin/bash
set -euo pipefail

# ==============================================================================
# Oxide Local Dev Setup
# Run inside WSL2 Ubuntu (or native Linux/macOS) from the repo root:
#   ./scripts/dev_setup.sh
#
# Brings up Postgres + Redis + Prometheus + Grafana via docker-compose.yml,
# applies migrations, and builds the workspace so `cargo run -p api` starts
# both the Axum API (:3001, also serves /metrics) and the Pingora proxy
# (:8000) — there is only one binary (`api`); the proxy is spawned
# in-process, there is no separate `proxy` binary. `telemetry` is a second
# binary (the Redis Streams ETL consumer) started separately.
# ==============================================================================

echo "== Oxide local dev setup =="

if grep -qi microsoft /proc/version 2>/dev/null; then
  echo ">>> Detected WSL2."
fi

# 1. Toolchain checks — install these yourself if missing, this script won't.
for bin in git cargo docker nix psql; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    echo "Missing required tool: $bin (see README prerequisites)" >&2
    exit 1
  fi
done

# 2. Docker daemon
if ! docker info >/dev/null 2>&1; then
  echo ">>> Starting docker daemon (may prompt for your sudo password)..."
  if command -v systemctl >/dev/null 2>&1 && systemctl is-system-running >/dev/null 2>&1; then
    sudo systemctl start docker
  else
    sudo service docker start
  fi
  for i in $(seq 1 15); do
    docker info >/dev/null 2>&1 && break
    sleep 1
  done
fi

# 3. Backing services: Postgres, Redis, Prometheus, Grafana
echo ">>> Bringing up docker-compose stack (postgres, redis, prometheus, grafana)..."
docker compose up -d

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/oxide"
export REDIS_URL="redis://127.0.0.1:6379"
if [ ! -f .env ]; then
  printf 'DATABASE_URL=%s\nREDIS_URL=%s\n' "$DATABASE_URL" "$REDIS_URL" > .env
fi

echo ">>> Waiting for Postgres to accept connections..."
until docker exec oxide-postgres pg_isready -U postgres >/dev/null 2>&1; do
  sleep 1
done

# 4. Migrations
if ! command -v sqlx >/dev/null 2>&1; then
  echo ">>> Installing sqlx-cli..."
  cargo install sqlx-cli --no-default-features --features postgres,rustls
fi
echo ">>> Running migrations..."
sqlx migrate run --database-url "$DATABASE_URL"

# 5. Build (debug — fast iteration; use --release for real benchmarks/demos)
echo ">>> Building workspace..."
cargo build

echo "== Done =="
echo "Start Oxide with:"
echo "  DATABASE_URL=$DATABASE_URL REDIS_URL=$REDIS_URL cargo run -p api"
echo "Start the telemetry ETL consumer with:"
echo "  DATABASE_URL=$DATABASE_URL REDIS_URL=$REDIS_URL cargo run -p telemetry"
echo ""
echo "API:        http://localhost:3001  (health at /health, metrics at /metrics)"
echo "Proxy:      http://localhost:8000"
echo "Prometheus: http://localhost:9090"
echo "Grafana:    http://localhost:3000  (anonymous admin access, 'Oxide Overview' dashboard)"
