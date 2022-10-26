use dotenv::dotenv;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic},
    config::FromClientConfig,
    ClientConfig,
};

use crate::{producer::KafkaProducer, NexmarkConfig};

pub struct NexmarkSource {
    producers: Vec<KafkaProducer>,
    client_config: ClientConfig,
    topic: String,
    num_partitions: usize,
}

impl NexmarkSource {
    pub fn new(conf: &NexmarkConfig) -> Self {
        dotenv().ok();
        let host = std::env::var("KAFKA_BROKER_URI").expect("specify a kafka broker to connect to");
        let topic =
            std::env::var("KAFKA_TOPIC").expect("specify a kafka topic to publish events to");
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", &host);
        let producers: Vec<KafkaProducer> = (0..conf.num_event_generators)
            .map(|i| KafkaProducer::new(client_config.clone(), topic.clone(), i))
            .collect();
        Self {
            producers: producers,
            client_config: client_config.to_owned(),
            topic,
            num_partitions: conf.num_event_generators,
        }
    }

    pub async fn create_topic(&self) {
        let admin_client = AdminClient::from_config(&self.client_config).unwrap();
        admin_client
            .create_topics(
                vec![&NewTopic::new(
                    &self.topic,
                    self.num_partitions as i32,
                    rdkafka::admin::TopicReplication::Fixed(1),
                )],
                &AdminOptions::new(),
            )
            .await
            .unwrap();
    }

    pub fn get_producer_for_generator(&self, generator_num: usize) -> &KafkaProducer {
        &self.producers[generator_num]
    }
}
