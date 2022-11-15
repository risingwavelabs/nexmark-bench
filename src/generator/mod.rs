use crate::generator::config::GeneratorConfig;
use crate::generator::nexmark::event::Event;

pub mod config;
pub mod nexmark;
pub mod source;

pub struct NexmarkGenerator {
    config: GeneratorConfig,
    local_events_so_far: u64,
    pub index: u64,
}

impl NexmarkGenerator {
    pub fn new(config: GeneratorConfig, index: u64) -> Self {
        Self {
            config,
            local_events_so_far: 0,
            index,
        }
    }

    pub fn next_event(&mut self) -> Option<Event> {
        let new_event_id = self.get_next_event_id();
        if new_event_id >= self.config.max_events {
            return None;
        }
        let (event, _) = Event::new(new_event_id as usize, &self.config.nexmark_config, 0);
        self.local_events_so_far += 1;
        Some(event)
    }

    pub fn get_next_event_id(&self) -> u64 {
        self.local_events_so_far * self.config.generator_num + self.index
    }
}
