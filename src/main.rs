use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use clap::Parser;
use core::time;
use env_logger::Env;
use log::info;
use rocket::routes;
use rocket::Config as RocketConfig;

use nexmark_server::generator::source::NexmarkSource;
use nexmark_server::parser::ServerConfig;
use nexmark_server::run_generators;
use nexmark_server::server::qps;
use nexmark_server::NexmarkInterval;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("nexmark_server=info")).init();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    let conf = ServerConfig::parse();
    info!("ServerConfig: {:?}", conf);
    if !validate_server_config(&conf) {
        panic!("Invalidate server config: {:?}", conf);
    }

    let nexmark_source = Arc::new(NexmarkSource::new(&conf));
    let interval = Arc::new(NexmarkInterval::new(&conf));
    match &conf.create_topic {
        true => tokio::time::timeout(time::Duration::from_secs(10), nexmark_source.create_topic())
            .await
            .map_err(|_| {
                anyhow::Error::msg(
                    "Timed out while creating topic. Ensure the infra is up and running",
                )
            })?,
        false => {
            let config = RocketConfig {
                address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                port: conf.listen_port,
                ..Default::default()
            };
            nexmark_source.check_topic_exist().await?;
            let rocket = rocket::custom(&config)
                .manage(Arc::clone(&interval))
                .manage(conf.clone())
                .mount("/nexmark", routes![qps])
                .ignite()
                .await
                .unwrap();
            let shutdown_handle = rocket.shutdown();
            tokio::spawn(async move { rocket.launch().await.unwrap() });
            run_generators(conf, nexmark_source, running.clone(), interval.clone()).await;
            shutdown_handle.notify();
            Ok(())
        }
    }
}

fn validate_server_config(conf: &ServerConfig) -> bool {
    conf.delay >= conf.delay_interval
        && conf.delay_proportion >= 0.0
        && conf.delay_proportion < 1.0
}
