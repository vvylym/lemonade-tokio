//! Proxy Port module
//!
use crate::prelude::*;

/// Proxy service trait
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProxyService: Send + Sync + 'static {
    /// Accept connections
    async fn accept_connections(&self, ctx: &Arc<Context>) -> Result<(), ProxyError>;
}

#[cfg(test)]
mockall::mock! {

    pub MockProxyServiceSuccess {}


    #[async_trait]
    impl ProxyService for MockProxyServiceSuccess {
        async fn accept_connections(&self, ctx: &Arc<Context>) -> Result<(), ProxyError> {
            Ok(())
        }
    }
}

#[cfg(test)]
mockall::mock! {

    pub MockProxyServiceError {}

    #[async_trait]
    impl ProxyService for MockProxyServiceError {
        async fn accept_connections(&self, ctx: &Arc<Context>) -> Result<(), ProxyError> {
            Err(ProxyError::Unexpected("proxy error".into()))
        }
    }
}
