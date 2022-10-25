use dotenv::dotenv;

use crate::producer::KafkaProducer;

pub struct NexmarkSource {
    pub producer: KafkaProducer,
}

impl NexmarkSource {
    pub fn new() -> Self {
        dotenv().ok();
        let host = std::env::var("KAFKA_BROKER_URI").expect("specify a kafka broker to connect to");
        let topic =
            std::env::var("KAFKA_TOPIC").expect("specify a kafka topic to publish events to");
        Self {
            producer: KafkaProducer::new(host, topic),
        }
    }
}
