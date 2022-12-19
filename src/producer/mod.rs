use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use rdkafka::error::KafkaError;
use rdkafka::producer::{BaseRecord, ProducerContext, ThreadedProducer};
use rdkafka::types::RDKafkaError;
use rdkafka::{ClientConfig, ClientContext, Message};

use crate::generator::nexmark::event::Event;
use crate::generator::source::EnvConfig;

const RETRY_BASE_INTERVAL_US: u64 = 100;
const RETRY_MAX_INTERVAL_US: u64 = 1000000;

pub struct KafkaProducer {
    pub producer: ThreadedProducer<ProduceCallbackLogger>,
    env_config: Arc<EnvConfig>,
    generator_num: usize,
    key: String,
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
        let key = format!("event-{}", generator_num);
        Self {
            producer,
            env_config,
            generator_num,
            key,
        }
    }

    pub async fn send_data_to_topic(&self, data: &Event) -> Result<()> {
        let payload = data.to_json();

        let mut timeout_us = RETRY_BASE_INTERVAL_US;
        while timeout_us <= RETRY_MAX_INTERVAL_US {
            let res = self.producer.send(
                BaseRecord::<std::string::String, std::string::String>::to(self.choose_topic(data))
                    .key(&self.key)
                    .partition(self.choose_partition())
                    .payload(&payload),
            );

            if let Err((e, _)) = res {
                if let KafkaError::MessageProduction(RDKafkaError::QueueFull) = e {
                    println!(
                        "[Warning] failed to send message to kafka, message: {:?}, err: {:?}, timeout_us for retry: {:?}",
                        payload, e, timeout_us
                    );
                    tokio::time::sleep(Duration::from_micros(timeout_us)).await;
                    timeout_us *= 2;
                    continue;
                } else {
                    return Err(anyhow::Error::new(e));
                }
            } else {
                return Ok(());
            }
        }
        Err(anyhow!("send_data_to_topic Timeout"))
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
