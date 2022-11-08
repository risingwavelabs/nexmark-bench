use core::time;
use std::sync::Arc;

use anyhow;
use clap::Parser;
use env_logger::Env;
use nexmark_server::{
    create_generators_for_config, generator::source::NexmarkSource, parser::NexmarkConfig,
};
use rand_chacha::ChaCha8Rng;

#[tokio::main]
async fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "error,warn,info")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);
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
