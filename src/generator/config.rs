use crate::generator::nexmark::config::NexmarkConfig;
use crate::generator::nexmark::properties::NexmarkProperties;

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub nexmark_config: NexmarkConfig,
    pub base_time: u64,
    pub max_events: u64,
    pub generator_num: u64,
    pub skip_person: bool,
    pub skip_auction: bool,
    pub skip_bid: bool,
}

impl GeneratorConfig {
    pub fn new(
        max_events: u64,
        base_time: u64,
        generator_num: u64,
        skip_event_types: String,
        amplify_factor: usize,
    ) -> Self {
        let properties = NexmarkProperties::default();
        let config = NexmarkConfig::from(properties, amplify_factor).unwrap();
        let max_events = match max_events {
            0 => u64::MAX,
            _ => max_events,
        };
        let skip_person = skip_event_types.contains("person");
        let skip_auction = skip_event_types.contains("auction");
        let skip_bid = skip_event_types.contains("bid");
        Self {
            nexmark_config: config,
            base_time,
            generator_num,
            max_events,
            skip_person,
            skip_auction,
            skip_bid,
        }
    }

    pub fn get_event_delay_microseconds(event_rate: usize, num_generators: usize) -> u64 {
        1_000_000.0 as u64 * num_generators as u64 / event_rate as u64
    }
}
