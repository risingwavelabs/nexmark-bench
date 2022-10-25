use crate::generator::events::Event;
use anyhow::Result;
use rdkafka::{
    producer::{BaseProducer, BaseRecord},
    ClientConfig,
};

// TODO: Polling the producer in a separate thread increases latency by around 10% for high QPS
// Find out how to fix this

// pub struct KafkaProducer {
//     producer: ThreadedProducer<ProduceCallbackLogger>,
//     topic: String,
// }

pub struct KafkaProducer {
    producer: BaseProducer,
    topic: String,
}

impl KafkaProducer {
    pub fn new(host: String, topic: String) -> Self {
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", &host)
            .create()
            .expect("Failed to create kafka producer");
        Self { producer, topic }
    }

    pub fn send_data_to_topic(&mut self, data: &Event, generator_num: &i32) -> Result<()> {
        let data = serde_json::to_string(data)?;
        self.producer
            .send(
                BaseRecord::<std::string::String, std::string::String>::to(&self.topic)
                    .key(&format!("event-{}", generator_num))
                    .payload(&data),
            )
            .map_err(|e| anyhow::Error::new(e.0))
    }
}

// TODO: Polling the producer increases latency by around 10% for high QPS

// struct ProduceCallbackLogger;
// impl ClientContext for ProduceCallbackLogger {}
// impl ProducerContext for ProduceCallbackLogger {
//     type DeliveryOpaque = ();
//     fn delivery(
//         &self,
//         delivery_result: &rdkafka::producer::DeliveryResult<'_>,
//         _delivery_opaque: Self::DeliveryOpaque,
//     ) {
//         if let Err(producer_err) = delivery_result {
//             let key: &str = producer_err.1.key_view().unwrap().unwrap();
//             println!(
//                 "failed to produce message with key {} - {}",
//                 key, producer_err.0,
//             );
//         };
//     }
// }
