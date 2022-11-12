use serde::Deserialize;

// TODO(Kexiang): make them configurable
#[derive(Clone, Debug, Default, Deserialize)]
pub struct NexmarkProperties {
    /// The event time gap will be like the time gap in the generated data, default false
    /// TODO(Kexiang): not supported for now
    pub use_real_time: bool,

    /// Minimal gap between two events, default 100000, so that the default max throughput is 10000
    pub min_event_gap_in_ns: u64,

    pub active_people: Option<usize>,

    pub in_flight_auctions: Option<usize>,

    pub out_of_order_group_size: Option<usize>,

    pub avg_person_byte_size: Option<usize>,

    pub avg_auction_byte_size: Option<usize>,

    pub avg_bid_byte_size: Option<usize>,

    pub hot_seller_ratio: Option<usize>,

    pub hot_auction_ratio: Option<usize>,

    pub hot_bidder_ratio: Option<usize>,

    pub hot_channel_ratio: Option<usize>,

    pub hot_first_event_id: Option<usize>,

    pub first_event_number: Option<usize>,

    pub num_categories: Option<usize>,

    pub auction_id_lead: Option<usize>,

    pub hot_seller_ratio_2: Option<usize>,

    pub hot_auction_ratio_2: Option<usize>,

    pub hot_bidder_ratio_2: Option<usize>,

    pub person_proportion: Option<usize>,

    pub auction_proportion: Option<usize>,

    pub bid_proportion: Option<usize>,

    pub first_auction_id: Option<usize>,

    pub first_person_id: Option<usize>,

    pub first_category_id: Option<usize>,

    pub person_id_lead: Option<usize>,

    pub sine_approx_steps: Option<usize>,

    pub base_time: Option<usize>,

    pub us_states: Option<String>,

    pub us_cities: Option<String>,

    pub first_names: Option<String>,

    pub last_names: Option<String>,

    pub rate_shape: Option<String>,

    pub rate_period: Option<usize>,

    pub first_event_rate: Option<usize>,

    pub events_per_sec: Option<usize>,

    pub next_event_rate: Option<usize>,

    pub us_per_unit: Option<usize>,

    pub threads: Option<usize>,
}
