use generator::NexmarkGenerator;
use generator::{config::GeneratorConfig, source::NexmarkSource};
use parser::NexmarkConfig;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::{self, Instant};
pub mod generator;
pub mod parser;
pub mod producer;
use std::sync::atomic::Ordering;

static SEED: u64 = 0;

struct IntervalState {
    interval: AtomicU64,
}

/// Creates generators from config options and sends events directly to kafka
pub async fn create_generators_for_config<'a, T>(
    nexmark_config: &NexmarkConfig,
    nexmark_source: &Arc<NexmarkSource>,
) where
    T: Rng + std::marker::Send,
{
    let wallclock_base_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let mut v = Vec::<tokio::task::JoinHandle<()>>::new();
    let interval_state = Arc::new(IntervalState {
        interval: AtomicU64::new(GeneratorConfig::get_delay_per_generator(nexmark_config) as u64),
    });
    let start_time = SystemTime::now();
    for generator_num in 0..nexmark_config.num_event_generators {
        let generator_config = GeneratorConfig::new(
            nexmark_config.clone(),
            wallclock_base_time,
            0,
            generator_num,
        );
        let source = Arc::clone(nexmark_source);
        let mut rng = ChaCha8Rng::seed_from_u64(SEED);
        let mut generator = NexmarkGenerator::new(generator_config.clone(), rng.clone(), source);
        let interval_state = interval_state.clone();
        let jh = tokio::task::spawn_blocking(move || {
            loop {
                let now = Instant::now();
                if generator_config.nexmark_config.dynamic_qps
                    && generator.get_next_event_id() % 10000 == 0
                {
                    interval_state.interval.store(
                        generate_dynamic_period(&generator_config.nexmark_config, &mut rng),
                        Ordering::Relaxed,
                    );
                }
                let next_event = generator.next_event();
                match &next_event {
                    Ok(e) => match e {
                        Some(next_e) => {
                            if let Err(err) = generator
                                .nexmark_source
                                .get_producer_for_generator(generator_num)
                                .send_data_to_topic(&next_e)
                            {
                                eprintln!("Error in sending event {:?}: {}", &next_e, &err);
                                continue;
                            }
                        }
                        None => break,
                    },
                    Err(err) => {
                        eprintln!("Error in generating event {:?}: {}", &next_event, &err)
                    }
                };
                while (now.elapsed().as_micros() as u64)
                    < interval_state.interval.load(Ordering::Relaxed)
                {
                    std::hint::spin_loop()
                }
            }
            generator
                .nexmark_source
                .get_producer_for_generator(generator_num)
                .producer
                .flush(time::Duration::new(5, 0));
        });
        v.push(jh);
    }
    for jh in v.into_iter() {
        jh.await.unwrap();
    }
    println!(
        "Delivered {} events in {:?}",
        nexmark_config.max_events,
        SystemTime::elapsed(&start_time).unwrap()
    );
}

fn generate_dynamic_period<T: Rng>(nexmark_config: &NexmarkConfig, rng: &mut T) -> u64 {
    let min_event_rate = 1 as u64;
    let max_event_rate = 2 * GeneratorConfig::get_delay_per_generator(nexmark_config) as u64;
    let dist = Uniform::from(min_event_rate..max_event_rate);
    dist.sample(rng)
}
