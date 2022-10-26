use crate::generator::events::Event;
use anyhow::Result;
use rdkafka::{
    producer::{BaseRecord, ProducerContext, ThreadedProducer},
    ClientConfig, ClientContext, Message,
};

pub struct KafkaProducer {
    pub producer: ThreadedProducer<ProduceCallbackLogger>,
    topic: String,
    partition_num: usize,
}

impl KafkaProducer {
    pub fn new(client_config: ClientConfig, topic: String, partition_num: usize) -> Self {
        let producer: ThreadedProducer<ProduceCallbackLogger> = client_config
            .create_with_context(ProduceCallbackLogger {})
            .expect("Failed to create kafka producer");
        Self {
            producer,
            topic,
            partition_num,
        }
    }

    pub fn send_data_to_topic(&self, data: &Event) -> Result<()> {
        let data = serde_json::to_string(data)?;
        self.producer
            .send(
                BaseRecord::<std::string::String, std::string::String>::to(&self.topic)
                    .key(&format!("event-{}", &self.partition_num))
                    .partition(0 as i32)
                    .payload(&data),
            )
            .map_err(|e| anyhow::Error::new(e.0))
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
