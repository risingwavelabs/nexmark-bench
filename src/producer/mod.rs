use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

use anyhow::Result;
use kafka::producer::{Producer, Record};

use crate::generator::NextEvent;
pub struct KafkaProducer {
    producer: Producer,
    topic: String,
}

impl KafkaProducer {
    pub fn new(hosts: Vec<String>, topic: String) -> Self {
        let producer = Producer::from_hosts(hosts).create().unwrap();
        Self { producer, topic }
    }

    pub fn send_data_to_topic(&mut self, data: &NextEvent) -> Result<()> {
        let wallclock_time_now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let event = serde_json::to_string(&data.event)?;
        let record = Record::from_value(&self.topic, event);
        if data.wallclock_timestamp > wallclock_time_now {
            sleep(Duration::from_millis(
                data.wallclock_timestamp - wallclock_time_now,
            ));
        };
        self.producer.send(&record).map_err(anyhow::Error::from)
    }
}
