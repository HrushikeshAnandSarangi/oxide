use std::sync::LazyLock;

use prometheus::{
    Encoder, Histogram, IntCounter, IntCounterVec, IntGauge, TextEncoder, register_histogram,
    register_int_counter, register_int_counter_vec, register_int_gauge,
};

pub static DEPLOYMENTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        "oxide_deployments_total",
        "Total number of deployment status transitions, labeled by status",
        &["status"]
    )
    .expect("failed to register oxide_deployments_total")
});

pub static BUILD_DURATION_SECONDS: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(
        "oxide_build_duration_seconds",
        "Time spent building a deployment via Nix, in seconds"
    )
    .expect("failed to register oxide_build_duration_seconds")
});

pub static ACTIVE_CONTAINERS: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!(
        "oxide_active_containers",
        "Number of containers currently running under Oxide"
    )
    .expect("failed to register oxide_active_containers")
});

pub static HEALTH_CHECK_FAILURES_TOTAL: LazyLock<IntCounter> = LazyLock::new(|| {
    register_int_counter!(
        "oxide_health_check_failures_total",
        "Total number of failed health checks against deployed containers"
    )
    .expect("failed to register oxide_health_check_failures_total")
});

/// `LazyLock` only registers a metric with Prometheus on first access — on a
/// freshly booted instance with zero deploys, none of the statics above have
/// been touched yet, so `/metrics` would come back empty instead of showing
/// zero-valued series. Call this once at startup so dashboards and alerting
/// see every metric from boot, not just after the first event.
pub fn init() {
    LazyLock::force(&BUILD_DURATION_SECONDS);
    LazyLock::force(&ACTIVE_CONTAINERS);
    LazyLock::force(&HEALTH_CHECK_FAILURES_TOTAL);
    for status in [
        "Queued",
        "Building",
        "BuildFailed",
        "ImageBuilding",
        "ContainerStarting",
        "Running",
        "Crashed",
        "Stopped",
    ] {
        DEPLOYMENTS_TOTAL.with_label_values(&[status]).reset();
    }
}

/// Renders all registered metrics in Prometheus text exposition format.
pub fn gather() -> Vec<u8> {
    let metric_families = prometheus::gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    let _ = encoder.encode(&metric_families, &mut buffer);
    buffer
}
