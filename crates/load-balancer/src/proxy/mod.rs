use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::broadcast::Receiver;

use crate::{error::*, state::AppState};

/// Handles an individual client connection by proxying it to a selected backend.
///
/// This function:
/// 1. Selects the best backend using the configured load balancing algorithm
/// 2. Increments the backend's connection count
/// 3. Establishes a connection to the selected backend
/// 4. Proxies data bidirectionally between client and backend
/// 5. Automatically decrements the connection count
///
pub async fn handle_proxy(
    app_state: Arc<AppState>,
    mut client_stream: TcpStream,
    mut shutdown_receiver: Receiver<()>,
) -> Result<()> {
    // TODO: Select the backend

    // TODO: Maybe record metrics

    // TODO: Connect to backend with timeout and shutdown awareness

    // TODO: Track connection start time for per-backend metrics

    // TODO: Perform bidirectional data proxying with shutdown awareness

    // TODO: Record per-backend connection duration

    Ok(())
}
