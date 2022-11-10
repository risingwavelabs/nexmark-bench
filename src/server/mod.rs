use std::sync::{atomic::Ordering, Arc};

use rocket::{post, response::status, serde::json::Json, State};
use serde::Deserialize;

use crate::{generator::config::GeneratorConfig, parser::NexmarkConfig, NexmarkInterval};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct QPSHandler {
    pub qps: usize,
}

#[post("/qps", data = "<qps_handler>")]
pub fn qps(
    qps_handler: Json<QPSHandler>,
    interval_state: &State<Arc<NexmarkInterval>>,
    conf_state: &State<NexmarkConfig>,
) -> status::Accepted<std::string::String> {
    interval_state.microseconds.store(
        GeneratorConfig::get_event_delay_microseconds(
            qps_handler.qps,
            conf_state.num_event_generators,
        ),
        Ordering::Relaxed,
    );
    status::Accepted(Some(format!("qps: {}", qps_handler.qps)))
}
