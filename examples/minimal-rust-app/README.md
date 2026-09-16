# minimal-rust-app

A deliberately trivial Rust HTTP server (std-only, no dependencies, responds `200 OK` on `:3000` — matching the port [runtime/src/container.rs](../../runtime/src/container.rs) hardcodes for every deployment). It exists to produce a real, reproducible number for [benchmarks.md](../../benchmarks.md)'s container-size comparison — not as an example of a real app to deploy.

## Reproduce the measurement

```bash
# 1. Build a fully static musl binary via Nix (pkgsStatic.rustPlatform.buildRustPackage)
nix build
ls -la result/bin/server   # ~550KB static binary

# 2. Build the minimal image: `FROM scratch` + just the static binary
docker build -f Dockerfile.nix -t oxide-minimal-nix ./result

# 3. Build the naive baseline: full Rust SDK image, build and run in the same stage
docker build -f Dockerfile.naive -t oxide-minimal-naive .

# Compare
docker images | grep oxide-minimal
```

`Dockerfile.nix` is also what `runtime/src/image.rs` expects if this were deployed through Oxide's real pipeline: the Nix build's `result/` output must contain a file literally named `Dockerfile` at its root (rename `Dockerfile.nix` → `Dockerfile` to try that).
