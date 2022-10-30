use std::sync::Arc;

use crate::generator::{events::Event, source::EnvConfig};
use anyhow::Result;
use rdkafka::{
    producer::{BaseRecord, ProducerContext, ThreadedProducer},
    ClientConfig, ClientContext, Message,
};

pub struct KafkaProducer {
    pub producer: ThreadedProducer<ProduceCallbackLogger>,
    env_config: Arc<EnvConfig>,
    generator_num: usize,
}

impl<'a> KafkaProducer {
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
        let data = serde_json::to_string(data)?;
        self.producer
            .send(
                BaseRecord::<std::string::String, std::string::String>::to(
                    &self.env_config.base_topic,
                )
                .key(&format!("event-{}", &self.generator_num))
                .partition(self.choose_partition())
                .payload(&data),
            )
            .map_err(|e| anyhow::Error::new(e.0))
    }

    fn choose_partition(&self) -> i32 {
        self.generator_num as i32 % self.env_config.num_partitions as i32
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
