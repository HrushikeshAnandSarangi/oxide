//! Benchmarks the `/metrics` hot path: every Prometheus scrape against the
//! running api process calls `controller::metrics::gather()`, which walks
//! the global registry and text-encodes it. This replaces a prior
//! placeholder benchmark that only measured a 1ns `tokio::time::sleep` and
//! didn't exercise any real Oxide code.
use controller::metrics;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_metrics_endpoint(c: &mut Criterion) {
    // Populate the registry so gather() has a realistic amount of data to
    // walk and encode, similar to a long-running instance.
    for status in ["Running", "BuildFailed", "Crashed"] {
        metrics::DEPLOYMENTS_TOTAL
            .with_label_values(&[status])
            .inc();
    }
    metrics::BUILD_DURATION_SECONDS.observe(12.5);
    metrics::ACTIVE_CONTAINERS.set(3);
    metrics::HEALTH_CHECK_FAILURES_TOTAL.inc();

    c.bench_function("metrics_gather_and_encode", |b| b.iter(metrics::gather));

    c.bench_function("deployment_counter_increment", |b| {
        b.iter(|| {
            metrics::DEPLOYMENTS_TOTAL
                .with_label_values(&["Running"])
                .inc()
        })
    });

    c.bench_function("build_duration_observe", |b| {
        b.iter(|| metrics::BUILD_DURATION_SECONDS.observe(12.5))
    });
}

criterion_group!(benches, bench_metrics_endpoint);
criterion_main!(benches);
