use crate::generator::config::GeneratorConfig;
use crate::generator::nexmark::event::Event;

pub mod config;
pub mod nexmark;
pub mod source;

pub struct NexmarkGenerator {
    config: GeneratorConfig,
    local_events_so_far: u64,
    index: u64,
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
        loop {
            let new_event_id = self.local_events_so_far * self.config.generator_num + self.index;
            if new_event_id >= self.config.max_events {
                return None;
            }
            if let Some((event, _)) = Event::new(
                new_event_id as usize,
                &self.config.nexmark_config,
                0,
                self.config.skip_person,
                self.config.skip_auction,
                self.config.skip_bid,
            ) {
                self.local_events_so_far += 1;
                return Some(event);
            }
            self.local_events_so_far += 1;
        }
    }
}
