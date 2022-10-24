use clap::Parser;
use dotenv::dotenv;
use generator::config::GeneratorConfig;
use generator::events::Event;
use generator::{NexmarkGenerator, NextEvent};
use nexmark_server::Config;
use producer::KafkaProducer;
use rand::{rngs::ThreadRng, Rng};
use receiver::Receiver;
use sender::Sender;
use std::thread::{self, sleep};
use std::time::{Duration, SystemTime};
use std::{
    collections::VecDeque,
    sync::mpsc::{self},
};
pub mod generator;
pub mod receiver;
pub mod sender;
use nexmark_server::Config as NexmarkConfig;
pub mod producer;

const BUFFER_SIZE: usize = 3;

fn main() {
    let conf = Config::parse();
    let mut producer = setup_kafka();
    let rec = NexmarkSource::new(conf);
    let mut delivered = 0;
    let start_time = SystemTime::now();
    for event in rec {
        if let Err(e) = producer.send_data_to_topic(&event) {
            eprintln!("Could not publish event {:?}: {}", event, e);
            continue;
        }
        if delivered % 1000 == 0 {
            println!(
                "Event {} at time {}",
                delivered,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            );
        }
        delivered += 1;
    }
    println!(
        "{:?} messages delivered in {:?}",
        &delivered,
        &start_time.elapsed().unwrap().as_secs_f64()
    )
}

fn setup_kafka() -> KafkaProducer {
    dotenv().ok();
    let host = std::env::var("KAFKA_BROKER_URI").expect("specify a kafka broker to connect to");
    let topic = std::env::var("KAFKA_TOPIC").expect("specify a kafka topic to publish events to");
    KafkaProducer::new(vec![host], topic)
}

struct Channel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new(size: usize) -> Self {
        let (send_channel, receive_channel) = mpsc::sync_channel::<VecDeque<T>>(BUFFER_SIZE);
        Channel {
            sender: Sender::new(size, send_channel),
            receiver: Receiver::new(receive_channel),
        }
    }
}

pub struct NexmarkSource {
    next_events_receiver: Receiver<NextEvent>,
}

impl NexmarkSource {
    pub fn new(nexmark_config: NexmarkConfig) -> Self {
        NexmarkSource::from_next_events(create_generators_for_config::<ThreadRng>(nexmark_config))
    }
    pub fn from_next_events(next_events_receiver: Receiver<NextEvent>) -> Self {
        Self {
            next_events_receiver: next_events_receiver,
        }
    }
}

// loop over all received NextEvents and unwraps them
impl Iterator for NexmarkSource {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        let next_event = self.next_events_receiver.recv().ok()?;
        let wallclock_time_now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if next_event.wallclock_timestamp > wallclock_time_now {
            sleep(Duration::from_millis(
                next_event.wallclock_timestamp - wallclock_time_now,
            ));
        };
        Some(next_event.event)
    }
}

/// Creates generators from config options and returns a single channel which contains all events from all generators
/// These output events will then be sent to kafka
fn create_generators_for_config<T>(nexmark_config: NexmarkConfig) -> Receiver<NextEvent>
where
    T: Rng + Default,
{
    let wallclock_base_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let buffer_size = nexmark_config.source_buffer_size;
    let mut next_event_channels: Vec<Receiver<NextEvent>> = (0..nexmark_config
        .num_event_generators)
        .map(|generator_num| {
            GeneratorConfig::new(
                nexmark_config.clone(),
                wallclock_base_time,
                0,
                generator_num,
            )
        })
        .map(|generator_config| {
            let mut channel = Channel::new(buffer_size);
            thread::Builder::new()
                .name(format!("generator-{}", generator_config.first_event_number))
                .spawn(move || {
                    let mut generator =
                        NexmarkGenerator::new(generator_config, T::default(), wallclock_base_time);
                    while let Ok(Some(event)) = generator.next_event() {
                        channel.sender.send(event).unwrap();
                    }
                    channel.sender.flush().unwrap();
                })
                .unwrap();
            channel.receiver
        })
        .collect();
    let mut collection_channel: Channel<NextEvent> = Channel::new(buffer_size);
    thread::Builder::new()
        .name("collection_channel".into())
        .spawn(move || {
            let mut completed_generators = 0;
            while completed_generators < next_event_channels.len() {
                for rec in &mut next_event_channels {
                    match rec.recv() {
                        Ok(e) => collection_channel.sender.send(e).unwrap(),
                        _ => {
                            completed_generators += 1;
                        }
                    }
                }
            }
            collection_channel.sender.flush().unwrap();
        })
        .unwrap();
    collection_channel.receiver
}
