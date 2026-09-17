//! Live end-to-end checks that a generated flake.nix actually builds with
//! real Nix. These shell out to the `nix` binary and take a while (musl
//! toolchain resolution, etc.), so they're `#[ignore]`d by default — run
//! explicitly with:
//!   cargo test -p builder --test live_flake_gen -- --ignored --nocapture

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use builder::detect::{self, Language};
use builder::flake_gen;

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("oxide-live-flake-gen-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_nix_build(dir: &Path) -> std::process::Output {
    Command::new("nix")
        .arg("build")
        .current_dir(dir)
        .output()
        .expect("failed to invoke nix")
}

#[test]
#[ignore = "shells out to a real `nix build`; run with --ignored"]
fn generated_rust_flake_builds_and_produces_a_working_dockerfile() {
    let dir = scratch_dir("rust");

    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "oxide-live-test-app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "server"
path = "src/main.rs"
"#,
    )
    .unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("src/main.rs"),
        r#"use std::net::TcpListener;
fn main() {
    let _listener = TcpListener::bind("0.0.0.0:3000").unwrap();
}
"#,
    )
    .unwrap();
    // A Cargo.lock is required by cargoLock.lockFile in the generated flake.
    let status = Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());
    fs::write(dir.join("Dockerfile"), "FROM anything\n").unwrap();

    assert!(detect::should_generate_flake(&dir));
    assert_eq!(detect::detect(&dir), Some(Language::Rust));

    let generated = flake_gen::generate(&dir, Language::Rust).unwrap();
    assert!(!generated.needs_hash_retry);

    let output = run_nix_build(&dir);
    assert!(
        output.status.success(),
        "nix build failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let dockerfile = fs::read_to_string(dir.join("result/Dockerfile")).unwrap();
    assert!(dockerfile.contains("FROM scratch"));
    assert!(dockerfile.contains("COPY bin/server /app"));
    assert!(dir.join("result/bin/server").is_file());
}
