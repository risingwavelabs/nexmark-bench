use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use dotenv::dotenv;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic};
use rdkafka::config::FromClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer, DefaultConsumerContext};
use rdkafka::{ClientConfig, ClientContext};
use serde::Deserialize;

use crate::parser::ServerConfig;
use crate::producer::KafkaProducer;

// send one message per topic without replication
const REPLICATION_FACTOR: i32 = 1;
const KAFKA_GET_METADATA_TIMEOUT: Duration = Duration::from_secs(1);

pub struct NexmarkSource {
    producers: Vec<KafkaProducer>,
    client_config: ClientConfig,
    env_config: Arc<EnvConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvConfig {
    kafka_host: String,
    pub num_partitions: i32,
    pub separate_topics: bool,
    pub base_topic: String,
    pub auction_topic: String,
    pub bid_topic: String,
    pub person_topic: String,
}

impl NexmarkSource {
    pub fn new(nexmark_config: &ServerConfig) -> Self {
        dotenv().ok();
        let env_config = Arc::new(NexmarkSource::load_env());
        println!("Kafka address: {:?}", env_config.kafka_host);
        let client_config = NexmarkSource::generate_client_config(&env_config.kafka_host);
        let producers: Vec<KafkaProducer> = (0..nexmark_config.num_event_generators)
            .map(|i| KafkaProducer::new(&client_config, Arc::clone(&env_config), i))
            .collect();
        Self {
            producers,
            client_config,
            env_config,
        }
    }

    fn generate_client_config(kafka_host: &str) -> ClientConfig {
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", kafka_host)
            .set("batch.size", "100000")
            .set("linger.ms", "0")
            .set("compression.type", "lz4")
            .set("acks", "0")
            .set("queue.buffering.max.kbytes", "1000000")
            .set("retries", "0");
        client_config
    }

    fn load_env() -> EnvConfig {
        dotenv().ok();
        match envy::from_env::<EnvConfig>() {
            Ok(config) => config,
            Err(err) => panic!("{:?}", err),
        }
    }

    async fn delete_topics<T: ClientContext>(&self, admin_client: &AdminClient<T>) {
        println!("Cleaning up...");
        admin_client
            .delete_topics(
                &[
                    self.env_config.person_topic.as_str(),
                    self.env_config.auction_topic.as_str(),
                    self.env_config.bid_topic.as_str(),
                    self.env_config.base_topic.as_str(),
                ],
                &AdminOptions::new(),
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    pub async fn create_topic(&self) {
        let admin_client = AdminClient::from_config(&self.client_config).unwrap();
        self.delete_topics(&admin_client).await;
        println!("Creating...");
        match self.env_config.separate_topics {
            true => admin_client
                .create_topics(
                    vec![
                        &NewTopic::new(
                            &self.env_config.person_topic,
                            self.env_config.num_partitions,
                            rdkafka::admin::TopicReplication::Fixed(REPLICATION_FACTOR),
                        ),
                        &NewTopic::new(
                            &self.env_config.auction_topic,
                            self.env_config.num_partitions,
                            rdkafka::admin::TopicReplication::Fixed(REPLICATION_FACTOR),
                        ),
                        &NewTopic::new(
                            &self.env_config.bid_topic,
                            self.env_config.num_partitions,
                            rdkafka::admin::TopicReplication::Fixed(REPLICATION_FACTOR),
                        ),
                    ],
                    &AdminOptions::new(),
                )
                .await
                .unwrap(),
            false => admin_client
                .create_topics(
                    vec![&NewTopic::new(
                        &self.env_config.base_topic,
                        self.env_config.num_partitions,
                        rdkafka::admin::TopicReplication::Fixed(REPLICATION_FACTOR),
                    )],
                    &AdminOptions::new(),
                )
                .await
                .unwrap(),
        };
    }

    pub async fn check_topic_exist(&self) -> Result<()> {
        let consumer: BaseConsumer = self
            .client_config
            .create_with_context(DefaultConsumerContext)
            .unwrap();
        if self.env_config.separate_topics {
            consumer.fetch_metadata(
                Some(self.env_config.person_topic.as_str()),
                KAFKA_GET_METADATA_TIMEOUT,
            )?;
            consumer.fetch_metadata(
                Some(self.env_config.auction_topic.as_str()),
                KAFKA_GET_METADATA_TIMEOUT,
            )?;
            consumer.fetch_metadata(
                Some(self.env_config.bid_topic.as_str()),
                KAFKA_GET_METADATA_TIMEOUT,
            )?;
        } else {
            consumer.fetch_metadata(
                Some(self.env_config.base_topic.as_str()),
                KAFKA_GET_METADATA_TIMEOUT,
            )?;
        }
        Ok(())
    }

    pub fn get_producer_for_generator(&self, generator_num: usize) -> &KafkaProducer {
        &self.producers[generator_num]
    }
}
