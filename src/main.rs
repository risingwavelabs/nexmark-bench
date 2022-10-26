use clap::Parser;
use nexmark_server::{create_generators_for_config, NexmarkConfig};
use rand::rngs::OsRng;
pub mod generator;
pub mod producer;

#[tokio::main]
async fn main() {
    let conf = NexmarkConfig::parse();
    let generators = create_generators_for_config::<OsRng>(&conf);
    generators.await;
}
