use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use tokio::time;

use crate::generator::NexmarkGenerator;
use crate::generator::{config::GeneratorConfig, source::NexmarkSource};
use crate::parser::NexmarkConfig;

pub mod generator;
pub mod parser;
pub mod producer;

static SEED: u64 = 0;

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
    let start_time = SystemTime::now();
    for generator_num in 0..nexmark_config.num_event_generators {
        let generator_config = GeneratorConfig::new(
            nexmark_config.clone(),
            wallclock_base_time,
            0,
            generator_num,
        );
        let source = Arc::clone(nexmark_source);
        let jh = tokio::spawn(async move {
            let rng = ChaCha8Rng::seed_from_u64(SEED);
            let delay = generator_config.inter_event_delay_microseconds;
            let mut interval = time::interval(Duration::from_micros(
                delay as u64 * generator_config.nexmark_config.num_event_generators as u64,
            ));
            let mut generator = NexmarkGenerator::new(generator_config.clone(), rng, source);
            loop {
                interval.tick().await;
                let next_event = generator.next_event();
                match &next_event {
                    Ok(e) => match e {
                        Some(next_e) => {
                            if let Err(err) = generator
                                .nexmark_source
                                .get_producer_for_generator(generator_num)
                                .send_data_to_topic(next_e)
                            {
                                eprintln!("Error in sending event {:?}: {}", &next_e, &err);
                                continue;
                            }
                        }
                        None => break,
                    },
                    Err(err) => eprintln!("Error in generating event {:?}: {}", &next_event, &err),
                };
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
