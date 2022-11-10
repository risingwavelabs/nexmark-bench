use std::sync::Arc;

use crate::{parser::NexmarkConfig, NexmarkInterval};

pub const FIRST_PERSON_ID: usize = 1000;
pub const FIRST_AUCTION_ID: usize = 1000;
pub const FIRST_CATEGORY_ID: usize = 10;

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub nexmark_config: NexmarkConfig,
    pub base_time: u64,
    pub first_event_id: u64,
    pub max_events: u64,
    pub first_event_number: usize,
    pub inter_event_delay: Arc<NexmarkInterval>,
}

impl GeneratorConfig {
    pub fn new(
        nexmark_config: NexmarkConfig,
        base_time: u64,
        first_event_id: u64,
        first_event_number: usize,
        inter_event_delay: Arc<NexmarkInterval>,
    ) -> Self {
        let max_events = match nexmark_config.max_events {
            0 => u64::MAX,
            _ => nexmark_config.max_events,
        };
        Self {
            nexmark_config,
            base_time,
            first_event_id,
            first_event_number,
            max_events,
            inter_event_delay,
        }
    }

    pub fn get_event_delay_microseconds(event_rate: usize, num_generators: usize) -> u64 {
        1_000_000.0 as u64 * num_generators as u64 / event_rate as u64
    }

    pub fn next_event_number(&self, num_events: u64) -> u64 {
        self.first_event_number as u64
            + num_events * self.nexmark_config.num_event_generators as u64
    }

    pub fn timestamp_for_event(&self, event_number: u64) -> u64 {
        self.base_time
            + (self
                .inter_event_delay
                .microseconds
                .load(std::sync::atomic::Ordering::Relaxed)
                * event_number as u64)
                / 1000
    }
}
