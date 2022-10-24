use clap::Parser;

// Number of yet-to-be-created people and auction ids allowed.
pub const PERSON_ID_LEAD: usize = 10;

#[derive(Clone, Debug, Parser)]
pub struct Config {
    #[clap(long, default_value = "1")]
    pub person_proportion: usize,

    #[clap(long, default_value = "46")]
    pub bid_proportion: usize,

    #[clap(long, default_value = "3")]
    pub auction_proportion: usize,

    #[clap(long, default_value = "500")]
    pub avg_auction_byte_size: usize,

    #[clap(long, default_value = "100")]
    pub avg_bid_byte_size: usize,

    #[clap(long, default_value = "200")]
    pub avg_person_byte_size: usize,

    /// #[clap(long, default_value = "10000000")]
    #[clap(long, default_value = "1")]
    pub first_event_rate: usize,

    #[clap(long, default_value = "2")]
    pub hot_auction_ratio: usize,

    #[clap(long, default_value = "4")]
    pub hot_bidders_ratio: usize,

    #[clap(long, default_value = "4")]
    pub hot_sellers_ratio: usize,

    /// 0 is unlimited.
    /// #[clap(long, default_value = "100000000")]
    #[clap(long, default_value = "100")]
    pub max_events: u64,

    /// Number of event generators
    #[clap(long, default_value = "2")]
    pub num_event_generators: usize,

    #[clap(long, default_value = "10000")]
    pub source_buffer_size: usize,

    #[clap(long, default_value = "1000")]
    pub num_active_people: usize,

    /// Average number of auctions which should be inflight at any time
    #[clap(long, default_value = "100")]
    pub num_in_flight_auctions: usize,
}

impl Config {
    pub fn total_proportion(&self) -> usize {
        self.person_proportion + self.auction_proportion + self.bid_proportion
    }
}
