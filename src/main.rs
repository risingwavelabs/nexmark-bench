use std::sync::Arc;

use clap::Parser;
use core::time;
use rand_chacha::ChaCha8Rng;

use nexmark_server::create_generators_for_config;
use nexmark_server::generator::source::NexmarkSource;
use nexmark_server::parser::NexmarkConfig;

#[tokio::main]
async fn main() {
    let conf = NexmarkConfig::parse();
    let nexmark_source = Arc::new(NexmarkSource::new(&conf));
    match &conf.create_topic {
        true => tokio::time::timeout(time::Duration::from_secs(10), nexmark_source.create_topic())
            .await
            .map_err(|_| {
                anyhow::Error::msg(
                    "Timed out while creating topic. Ensure the infra is up and running",
                )
            })
            .unwrap(),
        false => create_generators_for_config::<ChaCha8Rng>(&conf, &nexmark_source).await,
    }
}
