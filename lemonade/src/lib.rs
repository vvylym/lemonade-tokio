//! Lemonade library
//!
mod commands;
mod handlers;

use clap::Parser;
pub use commands::LemonadeCommands;
pub use handlers::{run_load_balancer, run_worker};

#[derive(Parser)]
#[command(name = "lemonade")]
#[command(about = "Lemonade load balancer and worker CLI", long_about = None)]
struct LemonadeCli {
    #[command(subcommand)]
    command: LemonadeCommands,
}

/// Run the Lemonade CLI
///
/// This function parses the CLI arguments and runs the appropriate command.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    lemonade_observability::init_tracing("lemonade", env!("CARGO_PKG_VERSION"))?;

    // Don't initialize tracing here - let each service (worker or load balancer)
    // initialize its own tracing with its specific service name and version
    let cli = LemonadeCli::parse();

    match cli.command {
        LemonadeCommands::Worker {
            framework,
            config,
            address,
            name,
            delay,
        } => run_worker(framework, config, address, name, delay).await?,
        LemonadeCommands::LoadBalancer { config } => run_load_balancer(config).await?,
    }

    Ok(())
}
