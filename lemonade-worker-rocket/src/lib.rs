//! Lemonade worker Rocket
//!
mod fairing;
mod handler;

use fairing::TracingFairing;
use handler::{health_handler, work_handler};
use lemonade_service::{AppState, config::Config};

/// Run the Rocket worker server
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with service name from config and worker package version
    lemonade_observability::init_tracing(
        "lemonade-worker-rocket",
        env!("CARGO_PKG_VERSION"),
        config.service_name(),
        config.otlp_endpoint(),
        config.otlp_protocol(),
    )?;

    // Initialize metrics
    lemonade_observability::init_metrics(
        "lemonade-worker-rocket",
        env!("CARGO_PKG_VERSION"),
        config.service_name(),
        config.otlp_endpoint(),
        config.otlp_protocol(),
    )?;

    let state = AppState::new(config);
    let addr = *state.config.listen_address().as_ref();
    let rocket_config = rocket::Config {
        address: addr.ip(),
        port: addr.port(),
        ..rocket::Config::default()
    };

    let _rocket = rocket::custom(&rocket_config)
        .attach(TracingFairing)
        .manage(state)
        .mount("/", rocket::routes![health_handler, work_handler])
        .launch()
        .await?;

    Ok(())
}
