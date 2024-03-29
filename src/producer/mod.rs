use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use log::info;
use log::warn;
use rdkafka::error::KafkaError;
use rdkafka::producer::{BaseRecord, ProducerContext, ThreadedProducer};
use rdkafka::types::RDKafkaError;
use rdkafka::{ClientConfig, ClientContext, Message};

use crate::generator::nexmark::event::Event;
use crate::generator::source::EnvConfig;

const RETRY_BASE_INTERVAL_US: u64 = 1000;
const RETRY_MAX_INTERVAL_US: u64 = 1000000;

pub struct KafkaProducer {
    pub producer: ThreadedProducer<ProduceCallbackLogger>,
    env_config: Arc<EnvConfig>,
    partition_idx: Option<i32>,
    key: String,
}

impl KafkaProducer {
    pub fn new(
        client_config: &ClientConfig,
        env_config: Arc<EnvConfig>,
        generator_idx: usize,
        generator_num: usize,
    ) -> Self {
        let producer: ThreadedProducer<ProduceCallbackLogger> = client_config
            .create_with_context(ProduceCallbackLogger {})
            .expect("Failed to create kafka producer");
        let key = format!("event-{}", generator_idx);
        let partition_idx = if generator_num as i32 % env_config.num_partitions == 0 {
            Some(generator_idx as i32 % env_config.num_partitions)
        } else {
            None
        };
        Self {
            producer,
            env_config,
            partition_idx,
            key,
        }
    }

    pub async fn send_data_to_topic(&self, data: &String, topic: &str) -> Result<()> {
        let mut timeout_us = RETRY_BASE_INTERVAL_US;
        while timeout_us <= RETRY_MAX_INTERVAL_US {
            let record = if let Some(partition) = self.partition_idx {
                BaseRecord::<std::string::String, std::string::String>::to(topic)
                    .key(&self.key)
                    .partition(partition)
                    .payload(data)
            } else {
                BaseRecord::<std::string::String, std::string::String>::to(topic).payload(data)
            };
            let res = self.producer.send(record);

            if let Err((e, _)) = res {
                if let KafkaError::MessageProduction(RDKafkaError::QueueFull) = e {
                    warn!(
                        "[Warning] failed to send message to kafka, message: {:?}, err: {:?}, timeout_us for retry: {:?}",
                        data, e, timeout_us
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

    pub fn choose_topic(&self, data: &Event) -> &str {
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

    pub fn serialize_event(&self, event: Event) -> String {
        event.to_json(!self.env_config.separate_topics)
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
            info!(
                "failed to produce message with key {} - {}",
                key, producer_err.0,
            );
        };
    }
}
