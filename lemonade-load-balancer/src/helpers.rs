//! Helpers module
//!

/// Spawn background task macro
#[macro_export]
macro_rules! spawn_background_handle {
    ($svc:expr, $ctx:expr) => {
        tokio::spawn({
            let s = $svc.clone();
            let c = $ctx.clone();
            async move {
                let _ = s.start(c).await;
                let _ = s.shutdown().await;
            }
        })
    };
}

/// Run background tasks with timeout macro
#[macro_export]
macro_rules! drain_and_run_handles_with_timeout {
    ($ctx:expr, $config_handle:expr, $health_handle:expr, $metrics_handle:expr, $accept_handle:expr) => {
        // drain active connections using shared registry (atomic)
        let ctx = $ctx.clone();
        let drain_handle = async {
            let connections = ctx.connection_registry();
            while connections.total() != 0 {
                if let Err(e) = $ctx.keep_alive().await {
                    dbg!("error occurs while keeping state alive: {}", e);
                }
            }
        };
        let _ = timeout($ctx.drain_timeout(), drain_handle).await;

        let timeout_duration = $ctx.background_handle_timeout();
        // running background tasks with their timeout
        let _ = timeout(timeout_duration, $config_handle).await;
        let _ = timeout(timeout_duration, $health_handle).await;
        let _ = timeout(timeout_duration, $metrics_handle).await;

        // running accept handle a timeout
        let _ = timeout($ctx.accept_handle_timeout(), $accept_handle).await;

        // drop state to ensure all background tasks are stopped
        drop($ctx);
    };
}

/// Wait for shutdown macro
#[macro_export]
macro_rules! wait_for_shutdown {
    ($shutdown_tx:expr) => {
        let shutdown_tx = $shutdown_tx.clone();
        {
            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                let _ = shutdown_tx.send(());
            });
        }

        // wait for shutdown broadcast
        let mut wait = $shutdown_tx.subscribe();
        let _ = wait.recv().await;
    };
}
