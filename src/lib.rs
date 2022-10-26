use clap::Parser;
use generator::NexmarkGenerator;
use generator::{config::GeneratorConfig, source::NexmarkSource};
use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time;
pub mod generator;
pub mod producer;

#[derive(Clone, Debug, Parser)]
pub struct NexmarkConfig {
    #[clap(long, default_value = "1")]
    pub person_proportion: usize,

    #[clap(long, default_value = "46")]
    pub bid_proportion: usize,

    #[clap(long, default_value = "3")]
    pub auction_proportion: usize,

    #[clap(long, default_value = "0")]
    pub avg_auction_byte_size: usize,

    #[clap(long, default_value = "0")]
    pub avg_bid_byte_size: usize,

    #[clap(long, default_value = "0")]
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
    #[clap(long, default_value = "3")]
    pub num_event_generators: usize,

    #[clap(long, default_value = "10000")]
    pub source_buffer_size: usize,

    #[clap(long, default_value = "1000")]
    pub num_active_people: usize,

    /// Average number of auctions which should be inflight at any time
    #[clap(long, default_value = "100")]
    pub num_in_flight_auctions: usize,
}

impl Default for NexmarkConfig {
    fn default() -> Self {
        NexmarkConfig {
            auction_proportion: 3,
            avg_auction_byte_size: 0,
            avg_bid_byte_size: 0,
            avg_person_byte_size: 0,
            bid_proportion: 46,
            first_event_rate: 100_000,
            hot_auction_ratio: 2,
            hot_bidders_ratio: 4,
            hot_sellers_ratio: 4,
            max_events: 1000,
            num_active_people: 1000,
            num_event_generators: 3,
            num_in_flight_auctions: 100,
            person_proportion: 1,
            source_buffer_size: 10_000,
        }
    }
}

impl NexmarkConfig {
    pub fn total_proportion(&self) -> usize {
        self.person_proportion + self.auction_proportion + self.bid_proportion
    }
}

/// Creates generators from config options and sends events directly to kafka
pub async fn create_generators_for_config<T>(nexmark_config: &NexmarkConfig)
where
    T: Rng + Default + std::marker::Send,
{
    let wallclock_base_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let nexmark_source = Arc::new(NexmarkSource::new(nexmark_config));
    nexmark_source.create_topic().await;
    let mut v = Vec::<tokio::task::JoinHandle<()>>::new();
    let start_time = SystemTime::now();
    for generator_num in 0..nexmark_config.num_event_generators {
        let generator_config = GeneratorConfig::new(
            nexmark_config.clone(),
            wallclock_base_time,
            0,
            generator_num,
        );
        let source = Arc::clone(&nexmark_source);
        let jh = tokio::spawn(async move {
            let delay = generator_config.inter_event_delay_microseconds;
            let mut interval = time::interval(Duration::from_micros(
                delay as u64 * generator_config.nexmark_config.num_event_generators as u64,
            ));
            let mut generator =
                NexmarkGenerator::new(generator_config.clone(), T::default(), source);
            loop {
                interval.tick().await;
                let next_event = generator.next_event();
                match &next_event {
                    Ok(e) => match e {
                        Some(next_e) => {
                            if let Err(err) = generator
                                .nexmark_source
                                .get_producer_for_generator(generator_num)
                                .send_data_to_topic(&next_e)
                            {
                                eprintln!("Error in sending event {:?}: {}", &next_e, &err);
                                continue;
                            }
                        }
                        None => break,
                    },
                    Err(err) => eprintln!("Error in generating event {:?}: {}", &next_event, &err),
                };
            }
            generator
                .nexmark_source
                .get_producer_for_generator(generator_num)
                .producer
                .flush(time::Duration::new(5, 0));
            println!(
                "Delivered {} events in {:?}",
                generator.events_counts_so_far,
                SystemTime::elapsed(&start_time).unwrap()
            );
        });
        v.push(jh);
    }
    for jh in v.into_iter() {
        jh.await.unwrap();
    }
}
