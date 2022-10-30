use std::sync::Arc;

use clap::Parser;
use nexmark_server::{
    create_generators_for_config, generator::source::NexmarkSource, NexmarkConfig,
};
use rand_chacha::ChaCha8Rng;
pub mod generator;
pub mod producer;

#[tokio::main]
async fn main() {
    let conf = NexmarkConfig::parse();
    let nexmark_source = Arc::new(NexmarkSource::new(&conf));
    match &conf.create_topic {
        true => nexmark_source.create_topic().await,
        false => create_generators_for_config::<ChaCha8Rng>(&conf, &nexmark_source).await,
    }
}
