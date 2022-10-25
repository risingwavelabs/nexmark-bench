use clap::Parser;
use nexmark_server::{create_generators_for_config, NexmarkConfig};
use rand::rngs::OsRng;
use std::time::SystemTime;
pub mod generator;
pub mod producer;

#[tokio::main]
async fn main() {
    let conf = NexmarkConfig::parse();
    let start_time = SystemTime::now();
    let generators = create_generators_for_config::<OsRng>(&conf);
    generators.await;
    println!(
        "Printed {} events in {:?}",
        conf.max_events,
        SystemTime::elapsed(&start_time).unwrap()
    )
}
