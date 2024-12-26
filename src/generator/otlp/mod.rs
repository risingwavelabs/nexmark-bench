use event::MetricsData;

pub mod event;

pub fn main() {
    let metrics = MetricsData::generate(1, 1);
    let json_string = serde_json::to_string_pretty(&metrics).unwrap();
    println!("{}", json_string);
}
