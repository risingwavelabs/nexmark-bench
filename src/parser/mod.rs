use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct ServerConfig {
    #[clap(long, default_value = "1000")]
    pub event_rate: usize,

    /// 0 is unlimited.
    #[clap(long, default_value = "100")]
    pub max_events: u64,

    /// Number of event generators
    #[clap(long, default_value = "3")]
    pub num_event_generators: usize,

    /// The port listening.
    #[clap(long, default_value = "8000")]
    pub listen_port: u16,

    /// The event type to skip, e.g. "auction,person" means only produce bid events.
    #[clap(long, default_value = "")]
    pub skip_event_types: String,

    /// If yes, only create topics(if there are topics with the same names, first delete them).
    /// If no, the nexmark-server run assuming the topics already exist.
    #[clap(long, short, action)]
    pub create_topic: bool,

    #[clap(long, default_value = "0")]
    pub delay: u64,

    /// must be smaller than delay
    #[clap(long, default_value = "0")]
    pub delay_interval: u64,

    /// range: [0.0, 1.0), 0 means no delay. 
    #[clap(long, default_value = "0")]
    pub delay_proportion: f64,

    #[clap(long, default_value = "1")]
    pub amplify_factor: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            max_events: 100,
            create_topic: false,
            event_rate: 1000,
            num_event_generators: 3,
            skip_event_types: String::from(""),
            listen_port: 8000,
            delay: 0,
            delay_interval: 0,
            amplify_factor: 1,
            delay_proportion: 0.0,
        }
    }
}
