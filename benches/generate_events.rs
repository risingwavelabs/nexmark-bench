use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use nexmark_server::create_generators_for_config;
use nexmark_server::generator::source::NexmarkSource;
use nexmark_server::parser::NexmarkConfig;
use nexmark_server::NexmarkInterval;
use rand_chacha::ChaCha8Rng;

fn event_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_generation");
    // vary the qps and monitor the time required to send 1,000 events
    for qps in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let mut nexmark_config = NexmarkConfig::default();
        nexmark_config.event_rate = *qps;
        let interval = Arc::new(NexmarkInterval::new(&nexmark_config));
        let running = Arc::new(AtomicBool::new(true));
        let nexmark_source = Arc::new(NexmarkSource::new(&nexmark_config));
        group.bench_with_input(BenchmarkId::from_parameter(qps), qps, |b, &_qps| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| {
                    create_generators_for_config::<ChaCha8Rng>(
                        &nexmark_config,
                        &nexmark_source,
                        Arc::clone(&running),
                        Arc::clone(&interval),
                    )
                });
        });
    }
    // vary the number of generators and monitor the time required to send 1,000 events
    // since the qps is 10_000, gold standard is 0.1s per iter
    for num_gen in 1..10 {
        let mut nexmark_config = NexmarkConfig::default();
        nexmark_config.num_event_generators = num_gen as usize;
        nexmark_config.event_rate = 10_000;
        let interval = Arc::new(NexmarkInterval::new(&nexmark_config));
        nexmark_config.max_events = 1_000;
        let nexmark_source = Arc::new(NexmarkSource::new(&nexmark_config));
        let running = Arc::new(AtomicBool::new(true));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_gen),
            &num_gen,
            |b, &_num_gen| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        create_generators_for_config::<ChaCha8Rng>(
                            &nexmark_config,
                            &nexmark_source,
                            Arc::clone(&running),
                            Arc::clone(&interval),
                        )
                    });
            },
        );
    }
    // vary the event byte size and monitor the time required to send 1,000 events
    // since the qps is 10_000, gold standard is 0.1s per iter
    for byte_size in [100, 200, 300, 400, 500] {
        let mut nexmark_config = NexmarkConfig::default();
        nexmark_config.event_rate = 10_000;
        nexmark_config.max_events = 1_000;
        nexmark_config.additional_auction_byte_size = byte_size;
        nexmark_config.additional_bid_byte_size = byte_size;
        nexmark_config.additional_person_byte_size = byte_size;
        let interval = Arc::new(NexmarkInterval::new(&nexmark_config));
        let nexmark_source = Arc::new(NexmarkSource::new(&nexmark_config));
        let running = Arc::new(AtomicBool::new(true));
        group.bench_with_input(
            BenchmarkId::from_parameter(byte_size),
            &byte_size,
            |b, &_byte_size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap())
                    .iter(|| {
                        create_generators_for_config::<ChaCha8Rng>(
                            &nexmark_config,
                            &nexmark_source,
                            Arc::clone(&running),
                            Arc::clone(&interval),
                        )
                    });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, event_generation);
criterion_main!(benches);
