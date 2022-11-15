use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::SystemTime;

use parser::ServerConfig;
use tokio::time::{self, Interval};

use crate::generator::NexmarkGenerator;
use crate::generator::{config::GeneratorConfig, source::NexmarkSource};

pub mod generator;
pub mod parser;
pub mod producer;
pub mod server;

// check the interval and refresh after these number of events have gone by
const INTERVAL_CHECK_EVENT_FREQUENCY: u64 = 100;

// log to stdout after these number of events have gone by
const PRINT_FREQUENCY: u64 = 10000;

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

    pub fn get_interval(&self) -> Interval {
        time::interval(time::Duration::from_micros(
            self.microseconds.load(Ordering::Relaxed),
        ))
    }

    // ignore an unnecessary interval allocation if the intervals are the same
    pub fn refresh_interval(&self, interval: &mut Interval) {
        if interval.period().as_micros() as u64 == self.microseconds.load(Ordering::Relaxed) {
            return;
        }
        *interval = time::interval(time::Duration::from_micros(
            self.microseconds.load(Ordering::Relaxed),
        ));
    }
}

pub struct Logger {}
impl Logger {
    pub fn new() -> Self {
        Self {}
    }
    pub fn log_event_rate(&self, system_time: SystemTime) {
        let elapse = SystemTime::elapsed(&system_time).unwrap();
        let rate = PRINT_FREQUENCY as f64 / elapse.as_micros() as f64 * 1_000_000_f64;
        println!(
            "Produced {} events in {:?}, generate_rate: {} row/s",
            PRINT_FREQUENCY, elapse, rate
        );
    }
}

// Creates generators from config options and sends events directly to kafka
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
    );
    for generator_idx in 0..server_config.num_event_generators {
        let generator_config = generator_config.clone();
        let running = running.clone();
        let source = nexmark_source.clone();
        let nexmark_interval = nexmark_interval.clone();
        let handler = tokio::spawn(async move {
            let mut generator = NexmarkGenerator::new(generator_config, generator_idx as u64);
            let mut interval = nexmark_interval.get_interval();
            loop {
                interval.tick().await;
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                if should_update_interval(&generator) {
                    nexmark_interval.refresh_interval(&mut interval);
                }
                let next_event = generator.next_event();
                match next_event {
                    Some(next_e) => {
                        if let Err(err) = source
                            .get_producer_for_generator(generator_idx)
                            .send_data_to_topic(&next_e)
                        {
                            eprintln!("Error in sending event {:?}: {}", &next_e, &err);
                            continue;
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
    println!(
        "Delivered {} events in {:?}",
        server_config.max_events,
        SystemTime::elapsed(&start_time).unwrap()
    );
}

// checks whether interval should be updated, by ALL threads
fn should_update_interval(generator: &NexmarkGenerator) -> bool {
    generator.get_next_event_id() % INTERVAL_CHECK_EVENT_FREQUENCY == generator.index
}
