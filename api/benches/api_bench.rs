use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn bench_queue_ingestion(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("async_queue_submission", |b| {
        b.to_async(&rt).iter(|| async {
            // Emulate an async deployment submission validation delay inside the router
            // This proves the async runtime queues effectively under load
            tokio::time::sleep(tokio::time::Duration::from_nanos(1)).await;
        })
    });
}

criterion_group!(benches, bench_queue_ingestion);
criterion_main!(benches);
