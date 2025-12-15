//! Lemonade worker Actix
//!
mod handler;

use actix_web::{App, HttpServer, web};
use handler::{health_handler, work_handler};
use lemonade_service::{AppState, config::Config};

/// Run the Actix worker server
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with service name from config and worker package version
    lemonade_observability::init_tracing(
        config.service_name(),
        env!("CARGO_PKG_VERSION"),
    )?;

    let state = AppState::new(config);
    let listen_addr = *state.config.listen_address().as_ref();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health_handler))
            .route("/work", web::get().to(work_handler))
    })
    .bind(listen_addr)?
    .run()
    .await?;

    Ok(())
}
