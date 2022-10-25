use crate::generator::{
    config::{FIRST_AUCTION_ID, FIRST_CATEGORY_ID, FIRST_PERSON_ID},
    NexmarkGenerator,
};

use super::Auction;
use anyhow::Result;
use rand::Rng;
use std::{
    cmp,
    mem::{size_of, size_of_val},
};

const NUM_CATEGORIES: usize = 5;
const HOT_SELLER_RATIO: usize = 100;

impl<R: Rng> NexmarkGenerator<R> {
    pub fn next_auction(
        &mut self,
        events_count_so_far: u64,
        event_id: u64,
        timestamp: u64,
    ) -> Result<Auction> {
        let id = self.last_base0_auction_id(event_id) + FIRST_AUCTION_ID as u64;
        let seller = match self.rng.gen_range(0..HOT_SELLER_RATIO) {
            0 => self.next_base0_person_id(event_id),
            _ => {
                (self.last_base0_person_id(event_id) / HOT_SELLER_RATIO as u64)
                    * HOT_SELLER_RATIO as u64
            }
        } + FIRST_PERSON_ID as u64;

        let category = FIRST_CATEGORY_ID + self.rng.gen_range(0..NUM_CATEGORIES);
        let initial_bid = self.next_price();

        let next_length_ms: u64 = self.next_auction_length_ms(events_count_so_far, timestamp);

        let item_name = self.next_string(20);
        let description = self.next_string(100);
        let reserve = initial_bid + self.next_price();

        let current_size = size_of::<u64>()
            + size_of_val(item_name.as_str())
            + size_of_val(description.as_str())
            + size_of::<usize>() * 3 // (initial_bid, reserve, category)
            + size_of::<u64>() * 2; // seller, expires
        Ok(Auction {
            id,
            item_name,
            description,
            initial_bid,
            reserve,
            date_time: timestamp,
            expires: timestamp + next_length_ms,
            seller,
            category,
            extra: self.next_extra(
                current_size,
                self.config.nexmark_config.avg_auction_byte_size,
            ),
        })
    }

    pub fn last_base0_auction_id(&self, event_id: u64) -> u64 {
        let mut epoch = event_id / self.config.nexmark_config.total_proportion() as u64;
        let mut offset = event_id % self.config.nexmark_config.total_proportion() as u64;
        let person_proportion = self.config.nexmark_config.person_proportion as u64;
        let auction_proportion = self.config.nexmark_config.auction_proportion as u64;
        if offset < person_proportion {
            epoch = match epoch.checked_sub(1) {
                Some(e) => e,
                None => return 0,
            };
            offset = auction_proportion - 1;
        } else if offset >= (person_proportion + auction_proportion) {
            offset = auction_proportion - 1;
        } else {
            offset -= person_proportion;
        }
        epoch * auction_proportion + offset
    }

    pub fn next_base0_auction_id(&mut self, next_event_id: u64) -> u64 {
        let min_auction = self
            .last_base0_auction_id(next_event_id)
            .saturating_sub(self.config.nexmark_config.num_in_flight_auctions as u64);
        let max_auction = self.last_base0_auction_id(next_event_id);
        min_auction + self.rng.gen_range(0..(max_auction - min_auction + 1))
    }

    fn next_auction_length_ms(&mut self, event_count_so_far: u64, timestamp: u64) -> u64 {
        let current_event_number = self.config.next_event_number(event_count_so_far);
        let num_events_for_auctions = (self.config.nexmark_config.num_in_flight_auctions
            * self.config.nexmark_config.total_proportion())
            / self.config.nexmark_config.auction_proportion;
        let future_auction = self
            .config
            .timestamp_for_event(current_event_number + num_events_for_auctions as u64);
        let horizon = future_auction.saturating_sub(timestamp);

        1 + self.rng.gen_range(0..cmp::max(horizon * 2, 1))
    }
}
