use std::sync::{atomic::Ordering, Arc};

use rocket::{post, response::status, serde::json::Json, State};
use serde::Deserialize;

use crate::NexmarkInterval;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct QPSHandler {
    pub qps: usize,
}

#[post("/qps", data = "<qps_handler>")]
pub fn qps(
    qps_handler: Json<QPSHandler>,
    state: &State<Arc<NexmarkInterval>>,
) -> status::Accepted<std::string::String> {
    state
        .microseconds
        .store(1_000_000 / qps_handler.qps as u64, Ordering::Relaxed);
    status::Accepted(Some(format!("qps: {}", qps_handler.qps)))
}
