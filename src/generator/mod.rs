use log::info;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::generator::config::GeneratorConfig;
use crate::generator::nexmark::event::Event;

use self::sample::Sampler;

pub mod config;
pub mod nexmark;
pub mod sample;
pub mod source;

pub struct NexmarkGenerator {
    config: GeneratorConfig,
    local_events_so_far: u64,
    index: u64,
    delay: u64,
    delay_proportion: f64,
    rng: SmallRng,
    sampler: Sampler,
}

impl NexmarkGenerator {
    pub fn new(
        config: GeneratorConfig,
        index: u64,
        delay: u64,
        delay_interval: u64,
        delay_proportion: f64,
        delay_pattern: &str,
        zipf_alpha: f64,
    ) -> Self {
        assert!(delay_interval <= delay);
        let local_events_so_far = 0;
        let rng = SmallRng::seed_from_u64(index as u64);
        let sampler = Sampler::new(delay_pattern, delay_interval, zipf_alpha);

        Self {
            config,
            local_events_so_far,
            index,
            delay,
            delay_proportion,
            rng,
            sampler,
        }
    }

    pub fn next_event(&mut self) -> Option<Event> {
        loop {
            let mut new_event_id =
                self.local_events_so_far * self.config.generator_num + self.index;

            if new_event_id >= self.config.max_events {
                return None;
            }

            let seed: f64 = self.rng.gen_range(0.0..1.0);

            if self.local_events_so_far == self.delay {
                info!(
                    "{}th generator reaches delay point; {}",
                    self.index, self.delay
                );
            }
            let mut delayed = false;
            if self.delay > 0
                && seed < self.delay_proportion
                && self.local_events_so_far > self.delay
            {
                delayed = true;
                let delay_interval = self.sampler.sample(&mut self.rng);
                new_event_id -= (self.delay - delay_interval) * self.config.generator_num;
            }

            if let Some((event, _)) = Event::new(
                new_event_id as usize,
                &self.config.nexmark_config,
                0,
                self.config.skip_person,
                self.config.skip_auction,
                self.config.skip_bid,
            ) {
                if !delayed {
                    self.local_events_so_far += 1;
                }
                return Some(event);
            }
            if !delayed {
                self.local_events_so_far += 1;
            }
        }
    }
}
