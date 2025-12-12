//! Channel Bundle module
//!
use crate::prelude::*;

/// Channel bundle (senders only) to allow hot-swap of channel capacities
#[derive(Debug, Clone)]
pub struct ChannelBundle {
    /// Metrics capacity
    metrics_cap: usize,
    /// Health capacity
    health_cap: usize,
    /// Metrics receiver
    metrics_tx: MpscSender<MetricsEvent>,
    /// Health receiver
    health_tx: MpscSender<HealthEvent>,
}

impl ChannelBundle {
    /// Create a new channel bundle
    #[must_use]
    pub fn new(
        metrics_cap: usize,
        health_cap: usize,
    ) -> (Self, MpscReceiver<MetricsEvent>, MpscReceiver<HealthEvent>) {
        let (mx_tx, mx_rx) = mpsc::channel(metrics_cap);
        let (h_tx, h_rx) = mpsc::channel(health_cap);
        (
            Self {
                metrics_cap,
                health_cap,
                metrics_tx: mx_tx,
                health_tx: h_tx,
            },
            mx_rx,
            h_rx,
        )
    }

    /// Check if senders are still valid (for detecting channel swaps)
    #[inline]
    pub fn is_valid(&self) -> bool {
        todo!()
    }

    /// Get metrics capacity
    #[inline]
    pub fn metrics_cap(&self) -> usize {
        self.metrics_cap
    }

    /// Get health capacity
    #[inline]
    pub fn health_cap(&self) -> usize {
        self.health_cap
    }

    /// Get metrics sender
    #[inline]
    pub fn metrics_sender(&self) -> MpscSender<MetricsEvent> {
        self.metrics_tx.clone()
    }

    /// Get health sender
    #[inline]
    pub fn health_sender(&self) -> MpscSender<HealthEvent> {
        self.health_tx.clone()
    }
}
