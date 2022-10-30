use super::super::NexmarkConfig;

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
    pub inter_event_delay_microseconds: f64,
}

impl GeneratorConfig {
    pub fn new(
        nexmark_config: NexmarkConfig,
        base_time: u64,
        first_event_id: u64,
        first_event_number: usize,
    ) -> Self {
        let inter_event_delay_microseconds = 1_000_000.0 / (nexmark_config.first_event_rate as f64);
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
            inter_event_delay_microseconds,
        }
    }

    pub fn next_event_number(&self, num_events: u64) -> u64 {
        self.first_event_number as u64
            + num_events * self.nexmark_config.num_event_generators as u64
    }

    pub fn timestamp_for_event(&self, event_number: u64) -> u64 {
        self.base_time + (self.inter_event_delay_microseconds * event_number as f64) as u64 / 1000
    }
}
