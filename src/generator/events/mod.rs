pub mod auctions;
pub mod bids;
pub mod people;
pub mod prices;
pub mod strings;

use anyhow::Result;
use arcstr::ArcStr;
use serde::Serialize;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Default)]
pub struct Person {
    pub id: u64,
    pub name: ArcStr,
    pub email_address: ArcStr,
    pub credit_card: ArcStr,
    pub city: ArcStr,
    pub state: ArcStr,
    pub date_time: ArcStr,
    pub extra: ArcStr,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Auction {
    pub id: u64,
    pub item_name: ArcStr,
    pub description: ArcStr,
    pub initial_bid: usize,
    pub reserve: usize,
    pub date_time: ArcStr,
    pub expires: ArcStr,
    pub seller: u64,
    pub category: usize,
    pub extra: ArcStr,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Bid {
    pub auction: u64,
    pub bidder: u64,
    pub price: usize,
    pub channel: ArcStr,
    pub url: ArcStr,
    pub date_time: ArcStr,
    pub extra: ArcStr,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Event {
    Person(Person),
    Auction(Auction),
    Bid(Bid),
}

impl Event {
    pub fn to_json(&self) -> Result<String> {
        match self {
            Event::Person(event) => Ok(serde_json::to_string(event)?),
            Event::Auction(event) => Ok(serde_json::to_string(event)?),
            Event::Bid(event) => Ok(serde_json::to_string(event)?),
        }
    }
}
