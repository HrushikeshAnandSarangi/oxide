//! Benchmarks `resolve_target`, the function on Pingora's `upstream_peer`
//! hot path — it runs once per incoming request to map a Host header to a
//! backend port. Measured at increasing route-table sizes to show how the
//! DashMap-backed ProxyState scales as more apps are deployed concurrently.
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use proxy::router::resolve_target;
use proxy::state::ProxyState;

fn seeded_state(route_count: usize) -> ProxyState {
    let state = ProxyState::new();
    for i in 0..route_count {
        state.add_route(format!("app-{i}"), 4000 + (i as u16 % 1000));
    }
    state
}

fn bench_resolve_target(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_target");
    for route_count in [1usize, 8, 64, 1_000, 10_000] {
        let state = seeded_state(route_count);
        let host = format!("app-{}.oxide.dev", route_count / 2);

        group.bench_with_input(
            BenchmarkId::from_parameter(route_count),
            &route_count,
            |b, _| b.iter(|| resolve_target(&host, &state)),
        );
    }
    group.finish();
}

fn bench_miss(c: &mut Criterion) {
    let state = seeded_state(1_000);
    c.bench_function("resolve_target_miss", |b| {
        b.iter(|| resolve_target("unregistered.oxide.dev", &state))
    });
}

criterion_group!(benches, bench_resolve_target, bench_miss);
criterion_main!(benches);
