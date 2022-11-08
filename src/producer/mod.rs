use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::generator::{events::Event, source::EnvConfig};
use crate::producer::Event::{Auction, Bid, Person};
use crate::LOG_FREQUENCY;
use anyhow::Result;
use log::info;
use rdkafka::{
    producer::{BaseRecord, ProducerContext, ThreadedProducer},
    ClientConfig, ClientContext, Message,
};

pub struct KafkaProducer {
    pub producer: ThreadedProducer<ProduceCallbackLogger>,
    env_config: Arc<EnvConfig>,
    generator_num: usize,
    events_delivery_tracker: Arc<EventsDeliveryTracker>,
}

impl KafkaProducer {
    pub fn new(
        client_config: &ClientConfig,
        env_config: Arc<EnvConfig>,
        generator_num: usize,
    ) -> Self {
        let producer: ThreadedProducer<ProduceCallbackLogger> = client_config
            .create_with_context(ProduceCallbackLogger {})
            .expect("Failed to create kafka producer");
        Self {
            producer,
            env_config,
            generator_num,
            events_delivery_tracker: Arc::new(EventsDeliveryTracker {
                events_delivered: AtomicU64::new(0),
            }),
        }
    }
    pub fn send_data_to_topic(&self, data: &Event) -> Result<()> {
        let payload = serde_json::to_string(data)?;
        let events_callback = Arc::clone(&self.events_delivery_tracker);
        self.producer
            .send(
                BaseRecord::<std::string::String, std::string::String, Arc<EventsDeliveryTracker>>::with_opaque_to(self.choose_topic(data), events_callback)
                    .key(&format!("event-{}", &self.generator_num))
                    .partition(self.choose_partition())
                    .payload(&payload),
            )
            .map_err(|e| anyhow::Error::new(e.0))
    }

    fn choose_partition(&self) -> i32 {
        self.generator_num as i32 % self.env_config.num_partitions as i32
    }

    fn choose_topic(&self, data: &Event) -> &str {
        match self.env_config.separate_topics {
            false => &self.env_config.base_topic,
            true => match *data {
                Person(_) => &self.env_config.person_topic,
                Auction(_) => &self.env_config.auction_topic,
                Bid(_) => &self.env_config.bid_topic,
            },
        }
    }
}

#[derive(Clone)]
pub struct ProduceCallbackLogger {}

pub struct EventsDeliveryTracker {
    events_delivered: AtomicU64,
}

impl ClientContext for ProduceCallbackLogger {}
impl ProducerContext for ProduceCallbackLogger {
    type DeliveryOpaque = Arc<EventsDeliveryTracker>;
    fn delivery(
        &self,
        delivery_result: &rdkafka::producer::DeliveryResult<'_>,
        _delivery_opaque: Self::DeliveryOpaque,
    ) {
        match delivery_result {
            Err(producer_err) => {
                let key: &str = producer_err.1.key_view().unwrap().unwrap();
                println!(
                    "failed to produce message with key {} - {}",
                    key, producer_err.0,
                );
            }
            Ok(_) => {
                let prev = _delivery_opaque
                    .events_delivered
                    .fetch_add(1, Ordering::SeqCst);
                if (prev + 1) % LOG_FREQUENCY as u64 == 0 {
                    info!("Delivered {:?} events", LOG_FREQUENCY)
                }
            }
        };
    }
}
