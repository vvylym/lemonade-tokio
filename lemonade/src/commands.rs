use clap::Subcommand;
use std::path::PathBuf;

/// Commands for the Lemonade CLI
///
/// This enum defines the available commands for the Lemonade CLI.
#[derive(Subcommand)]
pub enum LemonadeCommands {
    /// Run a worker server
    #[command(alias = "w")]
    Worker {
        /// Framework to use (actix, axum, hyper, rocket)
        #[arg(short = 'f', long = "framework", value_name = "FRAMEWORK")]
        framework: String,

        /// Path to configuration file (JSON or TOML)
        #[arg(short = 'c', long = "config", value_name = "CONFIG_FILE")]
        config: Option<PathBuf>,

        /// Listen address (e.g., localhost:8080)
        #[arg(short = 'a', long = "address", value_name = "LISTEN_ADDRESS")]
        address: Option<String>,

        /// Service name
        #[arg(short = 'n', long = "name", value_name = "SERVICE_NAME")]
        name: Option<String>,

        /// Work delay in milliseconds
        #[arg(short = 'd', long = "delay", value_name = "DELAY_MILLISECONDS")]
        delay: Option<u64>,
    },
    /// Run a load balancer
    #[command(alias = "lb")]
    LoadBalancer {
        /// Path to configuration file (JSON or TOML)
        #[arg(short = 'c', long = "config", value_name = "CONFIG_FILE")]
        config: Option<PathBuf>,
    },
}
