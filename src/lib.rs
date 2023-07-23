use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::SystemTime;

use log::{error, info};
use parser::ServerConfig;
use tokio::time;

use crate::generator::NexmarkGenerator;
use crate::generator::{config::GeneratorConfig, source::NexmarkSource};

pub mod generator;
pub mod parser;
pub mod producer;
pub mod server;

const INTERVAL_CHECK_FREQUENCY: f64 = 10.0;
const PRINT_FREQUENCY: f64 = 0.2;

#[derive(Debug)]
pub struct NexmarkInterval {
    pub microseconds: AtomicU64,
}

impl NexmarkInterval {
    pub fn new(config: &ServerConfig) -> Self {
        Self {
            microseconds: AtomicU64::new(GeneratorConfig::get_event_delay_microseconds(
                config.event_rate,
                config.num_event_generators,
            )),
        }
    }
}

/// Creates generators from config options and sends events directly to kafka
pub async fn run_generators(
    server_config: ServerConfig,
    nexmark_source: Arc<NexmarkSource>,
    running: Arc<AtomicBool>,
    nexmark_interval: Arc<NexmarkInterval>,
) {
    let wallclock_base_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let mut handlers = Vec::<tokio::task::JoinHandle<()>>::new();
    let start_time = SystemTime::now();
    let generator_config = GeneratorConfig::new(
        server_config.max_events,
        wallclock_base_time,
        server_config.num_event_generators as u64,
        server_config.skip_event_types,
        server_config.amplify_factor,
        server_config.event_rate_factor,
    );

    for generator_idx in 0..server_config.num_event_generators {
        let generator_config = generator_config.clone();
        let running = running.clone();
        let source = nexmark_source.clone();
        let atomic_interval_supplied = nexmark_interval.clone();

        let generator_delay = server_config.delay / server_config.num_event_generators as u64
            + if (generator_idx as u64)
                < (server_config.delay % server_config.num_event_generators as u64)
            {
                1
            } else {
                0
            };

        let generator_delay_interval = server_config.delay_interval
            / server_config.num_event_generators as u64
            + if (generator_idx as u64)
                < (server_config.delay_interval % server_config.num_event_generators as u64)
            {
                1
            } else {
                0
            };
        let pattern = server_config.delay_pattern.clone();
        let handler = tokio::spawn(async move {
            let mut generator = NexmarkGenerator::new(
                generator_config,
                generator_idx as u64,
                generator_delay,
                generator_delay_interval,
                server_config.delay_proportion,
                pattern.as_str(),
                server_config.zipf_alpha,
            );
            let mut interval = time::interval(time::Duration::from_micros(
                atomic_interval_supplied
                    .microseconds
                    .load(Ordering::Relaxed),
            ));
            let mut new_interval = atomic_interval_supplied
                .microseconds
                .load(Ordering::Relaxed);

            let mut loop_idx = 0;
            let mut check_idx =
                ((1_000_000 / (new_interval + 1)) as f64 / INTERVAL_CHECK_FREQUENCY).ceil() as u64;
            let mut print_idx =
                ((1_000_000 / (new_interval + 1)) as f64 / PRINT_FREQUENCY).ceil() as u64;
            let mut timestamp = SystemTime::now();

            loop {
                interval.tick().await;
                loop_idx += 1;
                // if ctrc has been received, terminate the thread
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                // update interval for controlling generating rate
                if loop_idx % check_idx == 0 {
                    // if the interval has been chanegd by a POST to /nexmark/qps, change interval in generator
                    new_interval = atomic_interval_supplied
                        .microseconds
                        .load(Ordering::Relaxed);
                    if interval.period().as_micros() as u64 != new_interval {
                        interval = time::interval(time::Duration::from_micros(new_interval));
                        check_idx = ((1_000_000 / (new_interval + 1)) as f64
                            / INTERVAL_CHECK_FREQUENCY)
                            .ceil() as u64;
                        print_idx = ((1_000_000 / (new_interval + 1)) as f64 / PRINT_FREQUENCY)
                            .ceil() as u64;
                    }
                }

                // print real-time rate information
                if loop_idx % print_idx == 0 {
                    let elapse = SystemTime::elapsed(&timestamp).unwrap();
                    let rate = print_idx as f64 / elapse.as_micros() as f64 * 1_000_000_f64;
                    info!(
                        "Producer_{} produce {} event in {:?}, generate_rate: {} row/s",
                        generator_idx, print_idx, elapse, rate
                    );
                    timestamp = SystemTime::now();
                }

                let next_event = generator.next_event();

                match next_event {
                    Some(next_e) => {
                        let producer = source.get_producer_for_generator(generator_idx);
                        let topic = producer.choose_topic(&next_e);
                        let payload = producer.serialize_event(next_e);
                        if let Err(err) = producer.send_data_to_topic(&payload, topic).await {
                            error!("Error in sending event {:?}: {}", payload, &err);
                        }
                    }
                    None => break,
                };
            }
            source
                .get_producer_for_generator(generator_idx)
                .producer
                .flush(time::Duration::new(5, 0));
        });
        handlers.push(handler);
    }
    for handler in handlers.into_iter() {
        handler.await.unwrap();
    }
    info!(
        "Delivered {} events in {:?}",
        server_config.max_events,
        SystemTime::elapsed(&start_time).unwrap()
    );
}
