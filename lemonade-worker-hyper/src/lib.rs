//! Lemonade worker Hyper
//!
mod handler;

use handler::handle_request;
use hyper::{server::conn::http1::Builder, service::service_fn};
use hyper_util::rt::TokioIo;
use lemonade_service::{AppState, config::Config};
use tokio::net::TcpListener;

/// Run the Hyper worker server
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with service name from config and worker package version
    lemonade_observability::init_tracing(
        config.service_name(),
        env!("CARGO_PKG_VERSION"),
    )?;

    let state = AppState::new(config);

    let listener = TcpListener::bind(state.config.listen_address().as_ref()).await?;
    println!(
        "Hyper worker listening on {}",
        state.config.listen_address().as_ref()
    );

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();

        tokio::task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| {
                let state = state.clone();
                handle_request(req, state)
            });

            if let Err(err) = Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
