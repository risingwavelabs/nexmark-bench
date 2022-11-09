use anyhow;
use clap::Parser;
use core::time;
use nexmark_server::generator::config::GeneratorConfig;
use nexmark_server::NexmarkInterval;
use nexmark_server::{
    create_generators_for_config, generator::source::NexmarkSource, parser::NexmarkConfig,
};
use rand_chacha::ChaCha8Rng;
use rocket::post;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{response::status, routes};
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU64};
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
    let interval = Arc::new(NexmarkInterval {
        microseconds: AtomicU64::new(
            conf.num_event_generators as u64
                * GeneratorConfig::get_event_delay_microseconds(&conf) as u64,
        ),
    });
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
                .manage(Arc::clone(&interval))
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
                Arc::clone(&interval),
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
fn qps(
    qps_handler: Json<QPSHandler>,
    state: &State<Arc<NexmarkInterval>>,
) -> status::Accepted<std::string::String> {
    state
        .microseconds
        .store(1_000_000 / qps_handler.qps as u64, Ordering::Relaxed);
    status::Accepted(Some(format!("qps: {}", qps_handler.qps)))
}
