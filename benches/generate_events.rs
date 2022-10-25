use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use nexmark_server::create_generators_for_config;
use nexmark_server::NexmarkConfig;
use rand::rngs::OsRng;

fn vary_qps(c: &mut Criterion) {
    let mut group = c.benchmark_group("vary_qps");
    for qps in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let mut nexmark_config = NexmarkConfig::default();
        group.bench_with_input(BenchmarkId::from_parameter(qps), qps, |b, &qps| {
            nexmark_config.first_event_rate = qps as usize;
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| create_generators_for_config::<OsRng>(&nexmark_config));
        });
    }
    group.finish();
}

criterion_group!(benches, vary_qps);
criterion_main!(benches);
