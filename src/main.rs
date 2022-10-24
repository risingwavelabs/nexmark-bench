use clap::Parser;
use dotenv::dotenv;
use generator::config::GeneratorConfig;
use generator::NexmarkGenerator;
use nexmark_server::Config;
use producer::KafkaProducer;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::thread::{self};
use std::time::SystemTime;
pub mod generator;
pub mod receiver;
pub mod sender;
use nexmark_server::Config as NexmarkConfig;
pub mod producer;

fn main() {
    let conf = Config::parse();
    let start_time = SystemTime::now();
    create_generators_for_config::<ThreadRng>(&conf, &start_time);
    println!(
        "Printed {} events in {:?}",
        conf.max_events,
        SystemTime::elapsed(&start_time).unwrap()
    )
}

pub struct NexmarkSource {
    producer: KafkaProducer,
}

impl NexmarkSource {
    pub fn new() -> Self {
        dotenv().ok();
        let host = std::env::var("KAFKA_BROKER_URI").expect("specify a kafka broker to connect to");
        let topic =
            std::env::var("KAFKA_TOPIC").expect("specify a kafka topic to publish events to");
        Self {
            producer: KafkaProducer::new(vec![host], topic),
        }
    }
}

/// Creates generators from config options and sends events directly to kafka
fn create_generators_for_config<T>(nexmark_config: &NexmarkConfig, start_time: &SystemTime)
where
    T: Rng + Default,
{
    let wallclock_base_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let mut v = Vec::<std::thread::JoinHandle<()>>::new();
    for generator_num in 0..nexmark_config.num_event_generators {
        let generator_config = GeneratorConfig::new(
            nexmark_config.clone(),
            wallclock_base_time,
            0,
            generator_num,
        );
        let jh = thread::Builder::new()
            .name(format!("generator-{}", generator_config.first_event_number))
            .spawn(move || {
                let mut generator = NexmarkGenerator::new(
                    generator_config,
                    T::default(),
                    wallclock_base_time,
                    NexmarkSource::new(),
                );
                while let Ok(Some(event)) = generator.next_event() {
                    if let Err(e) = generator.nexmark_source.producer.send_data_to_topic(&event) {
                        eprintln!("Could not generate event {:?}: {}", event, e)
                    }
                }
            })
            .unwrap();
        v.push(jh);
    }
    for jh in v.into_iter() {
        jh.join().unwrap();
        println!(
            "Ended the thread after {:?}",
            SystemTime::elapsed(&start_time).unwrap()
        )
    }
}
