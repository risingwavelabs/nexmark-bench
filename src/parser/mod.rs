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

    #[clap(long, short, action)]
    pub create_topic: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            max_events: 100,
            create_topic: false,
            event_rate: 1000,
            num_event_generators: 3,
            listen_port: 8000,
        }
    }
}
