use crate::generator::{
    config::{FIRST_AUCTION_ID, FIRST_PERSON_ID},
    NexmarkGenerator,
};

use super::{strings::next_string, Bid};
use arcstr::ArcStr;
use cached::Cached;
use rand::Rng;
use std::mem::size_of;

const HOT_AUCTON_RATIO: usize = 100;
const HOT_BIDDER_RATIO: usize = 100;
const HOT_CHANNELS_RATIO: usize = 100;

pub const CHANNELS_NUMBER: u32 = 10_000;

static HOT_CHANNELS: [ArcStr; 4] = [
    arcstr::literal!("Google"),
    arcstr::literal!("Facebook"),
    arcstr::literal!("Baidu"),
    arcstr::literal!("Apple"),
];
static HOT_URLS: [ArcStr; 4] = [
    arcstr::literal!("https://www.nexmark.com/googl/item.htm?query=1"),
    arcstr::literal!("https://www.nexmark.com/meta/item.htm?query=1"),
    arcstr::literal!("https://www.nexmark.com/bidu/item.htm?query=1"),
    arcstr::literal!("https://www.nexmark.com/aapl/item.htm?query=1"),
];

const BASE_URL_PATH_LENGTH: usize = 5;

impl<R: Rng> NexmarkGenerator<R> {
    // check the cache to get the url for the bid. 1/10 chance the url is returned as is
    fn get_new_channel_instance(&mut self, channel_number: u32) -> (ArcStr, ArcStr) {
        self.bid_channel_cache
            .cache_get_or_set_with(channel_number, || {
                let mut url = get_base_url(&mut self.rng);
                url = match self.rng.gen_range(0..10) {
                    9 => url,
                    _ => format!("{}&channel_id={}", url, channel_number.reverse_bits()).into(),
                };

                (format!("channel-{}", channel_number).into(), url)
            })
            .clone()
    }

    pub fn next_bid(&mut self, event_id: u64, timestamp: u64) -> Bid {
        let auction = match self
            .rng
            .gen_range(0..self.config.nexmark_config.hot_auction_ratio)
        {
            0 => self.next_base0_auction_id(event_id),
            _ => {
                (self.last_base0_auction_id(event_id) / HOT_AUCTON_RATIO as u64)
                    * HOT_AUCTON_RATIO as u64
            }
        } + FIRST_AUCTION_ID as u64;

        let bidder = match self
            .rng
            .gen_range(0..self.config.nexmark_config.hot_bidders_ratio)
        {
            0 => self.next_base0_person_id(event_id),
            _ => {
                (self.last_base0_person_id(event_id) / HOT_BIDDER_RATIO as u64)
                    * HOT_BIDDER_RATIO as u64
                    + 1
            }
        } + FIRST_PERSON_ID as u64;

        let price = self.next_price();

        let channel_number = self.rng.gen_range(0..CHANNELS_NUMBER);
        let (channel, url) = match self.rng.gen_range(1..=HOT_CHANNELS_RATIO) {
            HOT_CHANNELS_RATIO => self.get_new_channel_instance(channel_number),
            _ => {
                let hot_index = self.rng.gen_range(0..HOT_CHANNELS.len());
                (HOT_CHANNELS[hot_index].clone(), HOT_URLS[hot_index].clone())
            }
        };
        let current_size = size_of::<u64>() * 3 + size_of::<usize>();
        Bid {
            auction,
            bidder,
            price,
            channel,
            url,
            date_time: timestamp,
            extra: self.next_extra(
                current_size,
                self.config.nexmark_config.additional_bid_byte_size,
            ),
        }
    }
}

fn get_base_url<R: Rng>(rng: &mut R) -> ArcStr {
    arcstr::format!(
        "https://www.nexmark.com/{}/item.htm?query=1",
        next_string(rng, BASE_URL_PATH_LENGTH)
    )
}
