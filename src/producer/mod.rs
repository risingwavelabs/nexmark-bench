use anyhow::Result;
use kafka::producer::{Producer, Record};

use crate::generator::events::Event;
pub struct KafkaProducer {
    producer: Producer,
    topic: String,
}

impl KafkaProducer {
    pub fn new(hosts: Vec<String>, topic: String) -> Self {
        let producer = Producer::from_hosts(hosts).create().unwrap();
        Self { producer, topic }
    }

    pub fn send_data_to_topic(&mut self, data: &Event) -> Result<()> {
        let data = serde_json::to_string(&data)?;
        let record = Record::from_value(&self.topic, data);
        self.producer.send(&record).map_err(anyhow::Error::from)
    }
}
