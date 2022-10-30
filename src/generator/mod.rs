use std::sync::Arc;

use self::{
    config::GeneratorConfig,
    events::{bids::CHANNELS_NUMBER, Event},
    source::NexmarkSource,
};

use anyhow::Result;
use arcstr::ArcStr;
use cached::SizedCache;
use rand::Rng;

pub mod config;
pub mod events;
pub mod source;

pub struct NexmarkGenerator<R: Rng> {
    config: GeneratorConfig,
    rng: R,
    bid_channel_cache: SizedCache<u32, (ArcStr, ArcStr)>,
    pub events_counts_so_far: u64,
    pub nexmark_source: Arc<NexmarkSource>,
}

impl<R: Rng> NexmarkGenerator<R>
where
    R: Rng,
{
    pub fn new(config: GeneratorConfig, rng: R, nexmark_source: Arc<NexmarkSource>) -> Self {
        Self {
            config,
            rng,
            bid_channel_cache: SizedCache::with_size(CHANNELS_NUMBER as usize),
            events_counts_so_far: 0,
            nexmark_source: nexmark_source,
        }
    }

    // globally unique id which can identify events
    pub fn get_next_event_id(&self) -> u64 {
        self.config.first_event_id + self.config.next_event_number(self.events_counts_so_far)
    }

    pub fn has_next(&self) -> bool {
        self.get_next_event_id() < self.config.max_events
    }

    pub fn next_event(&mut self) -> Result<Option<Event>> {
        let new_event_id = self.get_next_event_id();
        if !self.has_next() {
            return Ok(None);
        }
        let (auction_proportion, person_proportion, total_proportion) = (
            self.config.nexmark_config.auction_proportion as u64,
            self.config.nexmark_config.person_proportion as u64,
            self.config.nexmark_config.total_proportion() as u64,
        );
        let rem = new_event_id % total_proportion;
        let event_timestamp = self
            .config
            .timestamp_for_event(self.config.next_event_number(self.events_counts_so_far));
        let event = if rem < person_proportion {
            Event::Person(self.next_person(new_event_id, event_timestamp))
        } else if rem < person_proportion + auction_proportion {
            Event::Auction(self.next_auction(
                self.events_counts_so_far,
                new_event_id,
                event_timestamp,
            )?)
        } else {
            Event::Bid(self.next_bid(new_event_id, event_timestamp))
        };
        self.events_counts_so_far += 1;
        Ok(Some(event))
    }
}
