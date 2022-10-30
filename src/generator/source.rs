use std::{sync::Arc, time::Duration};

use dotenv::dotenv;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic},
    config::FromClientConfig,
    ClientConfig, ClientContext,
};

use serde::Deserialize;

use crate::{producer::KafkaProducer, NexmarkConfig};

// send one message per topic without replication
const REPLICATION_FACTOR: i32 = 1;

pub struct NexmarkSource {
    producers: Vec<KafkaProducer>,
    client_config: ClientConfig,
    env_config: Arc<EnvConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    host: String,
    pub num_partitions: i32,
    pub separate_topics: bool,
    pub base_topic: String,
}

impl NexmarkSource {
    pub fn new(nexmark_config: &NexmarkConfig) -> Self {
        dotenv().ok();
        let env_config = Arc::new(NexmarkSource::load_env());
        let client_config = NexmarkSource::generate_client_config(&env_config.host);
        let producers: Vec<KafkaProducer> = (0..nexmark_config.num_event_generators)
            .map(|i| KafkaProducer::new(&client_config, Arc::clone(&env_config), i))
            .collect();
        Self {
            producers: producers,
            client_config: client_config.to_owned(),
            env_config: env_config,
        }
    }

    fn generate_client_config(host: &str) -> ClientConfig {
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", &host)
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
            Ok(config) => return config,
            Err(err) => panic!("{:?}", err),
        }
    }

    async fn delete_topics<T: ClientContext>(&self, admin_client: &AdminClient<T>) {
        println!("Cleaning up...");
        admin_client
            .delete_topics(
                &vec![
                    format!("{}-person", self.env_config.base_topic).as_str(),
                    format!("{}-auction", self.env_config.base_topic).as_str(),
                    format!("{}-bid", self.env_config.base_topic).as_str(),
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
                            format!("{}-person", self.env_config.base_topic).as_str(),
                            self.env_config.num_partitions,
                            rdkafka::admin::TopicReplication::Fixed(REPLICATION_FACTOR),
                        ),
                        &NewTopic::new(
                            format!("{}-auction", self.env_config.base_topic).as_str(),
                            self.env_config.num_partitions,
                            rdkafka::admin::TopicReplication::Fixed(REPLICATION_FACTOR),
                        ),
                        &NewTopic::new(
                            format!("{}-bid", self.env_config.base_topic).as_str(),
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
                        self.env_config.base_topic.as_str(),
                        self.env_config.num_partitions,
                        rdkafka::admin::TopicReplication::Fixed(REPLICATION_FACTOR),
                    )],
                    &AdminOptions::new(),
                )
                .await
                .unwrap(),
        };
    }

    pub fn get_producer_for_generator(&self, generator_num: usize) -> &KafkaProducer {
        &self.producers[generator_num]
    }
}