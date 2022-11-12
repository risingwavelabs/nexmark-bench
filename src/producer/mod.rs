use std::sync::Arc;

use anyhow::Result;
use rdkafka::producer::{BaseRecord, ProducerContext, ThreadedProducer};
use rdkafka::{ClientConfig, ClientContext, Message};

use crate::generator::nexmark::event::Event;
use crate::generator::source::EnvConfig;

pub struct KafkaProducer {
    pub producer: ThreadedProducer<ProduceCallbackLogger>,
    env_config: Arc<EnvConfig>,
    generator_num: usize,
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
        }
    }

    pub fn send_data_to_topic(&self, data: &Event) -> Result<()> {
        let payload = data.to_json();
        self.producer
            .send(
                BaseRecord::<std::string::String, std::string::String>::to(self.choose_topic(data))
                    .key(&format!("event-{}", &self.generator_num))
                    .partition(self.choose_partition())
                    .payload(&payload),
            )
            .map_err(|e| anyhow::Error::new(e.0))
    }

    fn choose_partition(&self) -> i32 {
        self.generator_num as i32 % self.env_config.num_partitions
    }

    fn choose_topic(&self, data: &Event) -> &str {
        if self.env_config.separate_topics {
            match *data {
                Event::Person(_) => &self.env_config.person_topic,
                Event::Auction(_) => &self.env_config.auction_topic,
                Event::Bid(_) => &self.env_config.bid_topic,
            }
        } else {
            &self.env_config.base_topic
        }
    }
}

#[derive(Clone)]
pub struct ProduceCallbackLogger;
impl ClientContext for ProduceCallbackLogger {}
impl ProducerContext for ProduceCallbackLogger {
    type DeliveryOpaque = ();
    fn delivery(
        &self,
        delivery_result: &rdkafka::producer::DeliveryResult<'_>,
        _delivery_opaque: Self::DeliveryOpaque,
    ) {
        if let Err(producer_err) = delivery_result {
            let key: &str = producer_err.1.key_view().unwrap().unwrap();
            println!(
                "failed to produce message with key {} - {}",
                key, producer_err.0,
            );
        };
    }
}
