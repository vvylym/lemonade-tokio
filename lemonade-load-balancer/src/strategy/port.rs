//! Strategy Port module
//!
use crate::prelude::*;

/// Strategy service trait
#[async_trait]
pub trait StrategyService: Send + Sync + 'static {
    /// Get the strategy
    fn strategy(&self) -> Strategy;
    /// Pick a backend, returns the selected backend metadata
    async fn pick_backend(&self, ctx: Arc<Context>)
    -> Result<BackendMeta, StrategyError>;
}
