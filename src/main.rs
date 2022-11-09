use anyhow;
use clap::Parser;
use core::time;
use nexmark_server::{
    create_generators_for_config, generator::source::NexmarkSource, parser::NexmarkConfig,
};
use rand_chacha::ChaCha8Rng;
use rocket::post;
use rocket::serde::json::Json;
use rocket::{response::status, routes};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use serde::Deserialize;

#[tokio::main]
async fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
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
        false => {
            let rocket = rocket::build()
                .mount("/nexmark", routes![qps])
                .ignite()
                .await
                .unwrap();
            let shutdown_handle = rocket.shutdown();
            tokio::spawn(async move { rocket.launch().await.unwrap() });
            create_generators_for_config::<ChaCha8Rng>(
                &conf,
                &nexmark_source,
                Arc::clone(&running),
            )
            .await;
            shutdown_handle.notify();
        }
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct QPSHandler {
    qps: usize,
}

#[post("/qps", data = "<qps_handler>")]
fn qps(qps_handler: Json<QPSHandler>) -> status::Accepted<String> {
    status::Accepted(Some(format!("qps: {}", qps_handler.qps)))
}
