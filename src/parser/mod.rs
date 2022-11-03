use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct NexmarkConfig {
    #[clap(long, default_value = "1")]
    pub person_proportion: usize,

    #[clap(long, default_value = "46")]
    pub bid_proportion: usize,

    #[clap(long, default_value = "3")]
    pub auction_proportion: usize,

    #[clap(long, default_value = "0")]
    pub additional_auction_byte_size: usize,

    #[clap(long, default_value = "0")]
    pub additional_bid_byte_size: usize,

    #[clap(long, default_value = "0")]
    pub additional_person_byte_size: usize,

    #[clap(long, default_value = "1_000")]
    pub event_rate: usize,

    #[clap(long, default_value = "2")]
    pub hot_auction_ratio: usize,

    #[clap(long, default_value = "4")]
    pub hot_bidders_ratio: usize,

    #[clap(long, default_value = "4")]
    pub hot_sellers_ratio: usize,

    /// 0 is unlimited.
    #[clap(long, default_value = "100")]
    pub max_events: u64,

    /// Number of event generators
    #[clap(long, default_value = "3")]
    pub num_event_generators: usize,

    #[clap(long, default_value = "10000")]
    pub source_buffer_size: usize,

    #[clap(long, default_value = "1000")]
    pub num_active_people: usize,

    #[clap(long, default_value = "100")]
    pub num_in_flight_auctions: usize,

    #[clap(long, short, action)]
    pub create_topic: bool,

    #[clap(long, short, action)]
    pub dynamic_qps: bool,
}

impl Default for NexmarkConfig {
    fn default() -> Self {
        NexmarkConfig {
            auction_proportion: 3,
            additional_auction_byte_size: 0,
            additional_bid_byte_size: 0,
            additional_person_byte_size: 0,
            bid_proportion: 46,
            event_rate: 1000,
            hot_auction_ratio: 2,
            hot_bidders_ratio: 4,
            hot_sellers_ratio: 4,
            max_events: 1000,
            num_active_people: 1000,
            num_event_generators: 3,
            num_in_flight_auctions: 100,
            person_proportion: 1,
            source_buffer_size: 10_000,
            create_topic: false,
            dynamic_qps: false,
        }
    }
}

impl NexmarkConfig {
    pub fn total_proportion(&self) -> usize {
        self.person_proportion + self.auction_proportion + self.bid_proportion
    }
}
