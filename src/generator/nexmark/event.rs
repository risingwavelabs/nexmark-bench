use std::cmp::{max, min};

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::generator::nexmark::config::{NexmarkConfig, CHANNEL_NUMBER};
use crate::generator::nexmark::utils::{milli_ts_to_timestamp_string, NexmarkRng};

type Id = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    Person,
    Auction,
    Bid,
}

/// The `Nexmark` Event, including `Person`, `Auction`, and `Bid`.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// The Person event.
    Person(Person),
    /// The Auction event.
    Auction(Auction),
    /// The Bid event.
    Bid(Bid),
}

impl Event {
    /// Creates a new event randomly.
    pub fn new(
        events_so_far: usize,
        nex: &NexmarkConfig,
        wall_clock_base_time: usize,
        skip_person: bool,
        skip_auction: bool,
        skip_bid: bool,
    ) -> Option<(Event, usize)> {
        let rem = nex.next_adjusted_event(events_so_far) % nex.proportion_denominator;
        if rem < nex.person_proportion {
            if skip_person {
                return None;
            }
        } else if rem < nex.person_proportion + nex.auction_proportion {
            if skip_auction {
                return None;
            }
        } else if skip_bid {
            return None;
        };
        Some(Self::inner_new(
            rem,
            events_so_far,
            nex,
            wall_clock_base_time,
        ))
    }

    fn inner_new(
        rem: usize,
        events_so_far: usize,
        nex: &NexmarkConfig,
        wall_clock_base_time: usize,
    ) -> (Event, usize) {
        let timestamp = nex.event_timestamp(nex.next_adjusted_event(events_so_far));
        let new_wall_clock_base_time = timestamp - nex.base_time + wall_clock_base_time;
        let id = nex.first_event_id + nex.next_adjusted_event(events_so_far);
        let mut rng = SmallRng::seed_from_u64(id as u64);
        let event = if rem < nex.person_proportion {
            Event::Person(Person::new(id, timestamp, &mut rng, nex))
        } else if rem < nex.person_proportion + nex.auction_proportion {
            Event::Auction(Auction::new(events_so_far, id, timestamp, &mut rng, nex))
        } else {
            Event::Bid(Bid::new(id, timestamp, &mut rng, nex))
        };
        (event, new_wall_clock_base_time)
    }

    pub fn to_json(&self) -> String {
        match self {
            Event::Person(p) => serde_json::to_string(p).unwrap(),
            Event::Auction(a) => serde_json::to_string(a).unwrap(),
            Event::Bid(b) => serde_json::to_string(b).unwrap(),
        }
    }
}

/// Person represents a person submitting an item for auction and/or making a
/// bid on an auction.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    /// A person-unique integer ID.
    pub id: Id,
    /// A string for the person’s full name.
    pub name: String,
    /// The person’s email address as a string.
    pub email_address: String,
    /// The credit card number as a 19-letter string.
    pub credit_card: String,
    /// One of several US city names as a string.
    pub city: String,
    /// One of several US states as a two-letter string.
    pub state: String,
    /// A millisecond timestamp for the event origin.
    pub date_time: String,
    /// Extra information
    pub extra: String,
}

impl Person {
    /// Creates a new `Person` event.
    fn new(id: usize, time: usize, rng: &mut SmallRng, nex: &NexmarkConfig) -> Self {
        let id = Self::last_id(id, nex) + nex.first_person_id;
        let name = format!(
            "{} {}",
            nex.first_names.choose(rng).unwrap(),
            nex.last_names.choose(rng).unwrap(),
        );
        let email_address = format!("{}@{}.com", rng.gen_string(7), rng.gen_string(5));
        let credit_card = (0..4)
            .map(|_| format!("{:04}", rng.gen_range(0..10000)))
            .collect::<Vec<String>>()
            .join(" ");
        let city = nex.us_cities.choose(rng).unwrap().clone();
        let state = nex.us_states.choose(rng).unwrap().clone();

        let current_size =
            8 + name.len() + email_address.len() + credit_card.len() + city.len() + state.len();
        let extra = rng.gen_next_extra(current_size, nex.avg_person_byte_size);

        Self {
            id,
            name,
            email_address,
            credit_card,
            city,
            state,
            date_time: milli_ts_to_timestamp_string(time),
            extra,
        }
    }

    fn next_id(id: usize, rng: &mut SmallRng, nex: &NexmarkConfig) -> Id {
        let people = Self::last_id(id, nex) + 1;
        let active = min(people, nex.active_people);
        people - active + rng.gen_range(0..active + nex.person_id_lead)
    }

    fn last_id(id: usize, nex: &NexmarkConfig) -> Id {
        let epoch = id / nex.proportion_denominator;
        let mut offset = id % nex.proportion_denominator;
        if nex.person_proportion <= offset {
            offset = nex.person_proportion - 1;
        }
        epoch * nex.person_proportion + offset
    }
}

/// Auction represents an item under auction.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Auction {
    /// An auction-unique integer ID.
    pub id: Id,
    /// The name of the item being auctioned.
    pub item_name: String,
    /// A short description of the item.
    pub description: String,
    /// The initial bid price in cents.
    pub initial_bid: usize,
    /// The minimum price for the auction to succeed.
    pub reserve: usize,
    /// A millisecond timestamp for the event origin.
    pub date_time: String,
    /// A UNIX epoch timestamp for the expiration date of the auction.
    pub expires: String,
    /// The ID of the person that created this auction.
    pub seller: Id,
    /// The ID of the category this auction belongs to.
    pub category: Id,
    /// Extra information
    pub extra: String,
}

impl Auction {
    fn new(
        events_so_far: usize,
        id: usize,
        time: usize,
        rng: &mut SmallRng,
        nex: &NexmarkConfig,
    ) -> Self {
        let id = Self::last_id(id, nex) + nex.first_auction_id;
        let item_name = rng.gen_string(20);
        let description = rng.gen_string(100);
        let initial_bid = rng.gen_price();

        let reserve = initial_bid + rng.gen_price();
        let date_time = milli_ts_to_timestamp_string(time);
        let expires =
            milli_ts_to_timestamp_string(time + Self::next_length(events_so_far, rng, time, nex));
        let mut seller = if rng.gen_range(0..nex.hot_seller_ratio) > 0 {
            (Person::last_id(id, nex) / nex.hot_seller_ratio_2) * nex.hot_seller_ratio_2
        } else {
            Person::next_id(id, rng, nex)
        };
        seller += nex.first_person_id;
        let category = nex.first_category_id + rng.gen_range(0..nex.num_categories);

        let current_size = 8 + item_name.len() + description.len() + 8 + 8 + 8 + 8 + 8;
        let extra = rng.gen_next_extra(current_size, nex.avg_auction_byte_size);

        Auction {
            id,
            item_name,
            description,
            initial_bid,
            reserve,
            date_time,
            expires,
            seller,
            category,
            extra,
        }
    }

    fn next_id(id: usize, rng: &mut SmallRng, nex: &NexmarkConfig) -> Id {
        let max_auction = Self::last_id(id, nex);
        let min_auction = if max_auction < nex.in_flight_auctions {
            0
        } else {
            max_auction - nex.in_flight_auctions
        };
        min_auction + rng.gen_range(0..max_auction - min_auction + 1 + nex.auction_id_lead)
    }

    fn last_id(id: usize, nex: &NexmarkConfig) -> Id {
        let mut epoch = id / nex.proportion_denominator;
        let mut offset = id % nex.proportion_denominator;
        if offset < nex.person_proportion {
            epoch -= 1;
            offset = nex.auction_proportion - 1;
        } else if nex.person_proportion + nex.auction_proportion <= offset {
            offset = nex.auction_proportion - 1;
        } else {
            offset -= nex.person_proportion;
        }
        epoch * nex.auction_proportion + offset
    }

    fn next_length(
        events_so_far: usize,
        rng: &mut SmallRng,
        time: usize,
        nex: &NexmarkConfig,
    ) -> usize {
        let current_event = nex.next_adjusted_event(events_so_far);
        let events_for_auctions =
            (nex.in_flight_auctions * nex.proportion_denominator) / nex.auction_proportion;
        let future_auction = nex.event_timestamp(current_event + events_for_auctions);

        let horizon = future_auction - time;
        1 + rng.gen_range(0..max(horizon * 2, 1))
    }
}

/// Bid represents a bid for an item under auction.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Bid {
    /// The ID of the auction this bid is for.
    pub auction: Id,
    /// The ID of the person that placed this bid.
    pub bidder: Id,
    /// The price in cents that the person bid for.
    pub price: usize,
    /// The channel of this bid
    pub channel: String,
    /// The url of this bid
    pub url: String,
    /// A millisecond timestamp for the event origin.
    pub date_time: String,
    /// Extra information
    pub extra: String,
}

impl Bid {
    fn new(id: usize, time: usize, rng: &mut SmallRng, nex: &NexmarkConfig) -> Self {
        let auction = if 0 < rng.gen_range(0..nex.hot_auction_ratio) {
            (Auction::last_id(id, nex) / nex.hot_auction_ratio_2) * nex.hot_auction_ratio_2
        } else {
            Auction::next_id(id, rng, nex)
        };
        let bidder = if 0 < rng.gen_range(0..nex.hot_bidder_ratio) {
            (Person::last_id(id, nex) / nex.hot_bidder_ratio_2) * nex.hot_bidder_ratio_2 + 1
        } else {
            Person::next_id(id, rng, nex)
        };

        let price = rng.gen_price();

        let (channel, url) = if rng.gen_range(0..nex.hot_channel_ratio) > 0 {
            let index = rng.gen_range(0..nex.hot_channels.len());
            (nex.hot_channels[index].clone(), nex.hot_urls[index].clone())
        } else {
            nex.channel_url_map
                .get(&rng.gen_range(0..CHANNEL_NUMBER))
                .unwrap()
                .clone()
        };

        let current_size = 8 + 8 + 8 + 8;
        let extra = rng.gen_next_extra(current_size, nex.avg_bid_byte_size);

        Bid {
            auction: auction + nex.first_auction_id,
            bidder: bidder + nex.first_person_id,
            price,
            date_time: milli_ts_to_timestamp_string(time),
            channel,
            url,
            extra,
        }
    }
}
