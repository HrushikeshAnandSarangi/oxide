//! Benchmarks the AES-256-GCM encrypt/decrypt path used for project
//! environment variables (see deployment_service.rs, which decrypts every
//! configured env var on each deploy).
use common::crypto::{decrypt, encrypt};
use criterion::{Criterion, criterion_group, criterion_main};

const KEY: [u8; 32] = [7u8; 32];
const SHORT_VALUE: &str = "production";
const LONG_VALUE: &str = "postgres://oxide_user:a-fairly-long-password-value@db.internal.oxide:5432/oxide_prod?sslmode=require&application_name=oxide-api";

fn bench_crypto(c: &mut Criterion) {
    c.bench_function("encrypt_short_env_var", |b| {
        b.iter(|| encrypt(SHORT_VALUE, &KEY).unwrap())
    });

    c.bench_function("encrypt_long_env_var", |b| {
        b.iter(|| encrypt(LONG_VALUE, &KEY).unwrap())
    });

    let (short_ct, short_nonce) = encrypt(SHORT_VALUE, &KEY).unwrap();
    c.bench_function("decrypt_short_env_var", |b| {
        b.iter(|| decrypt(&short_ct, &short_nonce, &KEY).unwrap())
    });

    let (long_ct, long_nonce) = encrypt(LONG_VALUE, &KEY).unwrap();
    c.bench_function("decrypt_long_env_var", |b| {
        b.iter(|| decrypt(&long_ct, &long_nonce, &KEY).unwrap())
    });
}

criterion_group!(benches, bench_crypto);
criterion_main!(benches);
