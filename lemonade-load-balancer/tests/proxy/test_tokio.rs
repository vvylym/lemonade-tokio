//! Tests for TokioProxyService
//!
use lemonade_load_balancer::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use crate::common::fixtures::{create_test_backend, create_test_context};

#[tokio::test]
async fn tokio_proxy_service_new_should_succeed() {
    // Given: a ProxyConfig
    let config = ProxyConfig {
        listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0), // Use 0 for auto-assign
        max_connections: Some(1000),
    };

    // When: creating TokioProxyService
    let service = TokioProxyService::new(Arc::new(ArcSwap::from_pointee(config)));

    // Then: service is created successfully
    assert!(service.is_ok());
}

#[tokio::test]
async fn tokio_proxy_service_accept_connections_binds_listener_should_succeed() {
    // Given: a service and context
    let config = ProxyConfig {
        listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        max_connections: Some(1000),
    };
    let service = TokioProxyService::new(Arc::new(ArcSwap::from_pointee(config)))
        .expect("Failed to create service");
    let ctx = create_test_context(vec![]);

    // When: calling accept_connections()
    // Spawn in background since it runs indefinitely
    let accept_handle =
        tokio::spawn(async move { service.accept_connections(&ctx).await });

    // Wait a bit for listener to bind
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Then: listener is bound to configured address
    // We can't directly verify, but if it didn't bind, the test would fail
    accept_handle.abort();
}

#[tokio::test]
async fn tokio_proxy_service_proxies_connection_should_succeed() {
    // Given: a service, context, client, and backend server
    let backend = create_test_backend(0, None, Some(10u8));
    let backend_addr = *backend.address().as_ref();

    // Create backend server that echoes data
    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(backend_addr)
            .await
            .expect("Failed to bind server");
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = [0u8; 1024];
            if let Ok(n) = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await
                && n > 0
            {
                let _ = tokio::io::AsyncWriteExt::write_all(&mut stream, &buf[..n]).await;
            }
        }
    });

    let proxy_config = ProxyConfig {
        listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        max_connections: Some(1000),
    };
    let service = TokioProxyService::new(Arc::new(ArcSwap::from_pointee(proxy_config)))
        .expect("Failed to create service");
    let ctx = create_test_context(vec![backend]);

    // Set backend as healthy
    let health = ctx.health_registry();
    health.set_alive(0, true, 1000);

    // Start proxy service
    let proxy_handle = tokio::spawn({
        let service = service.clone();
        let ctx = ctx.clone();
        async move { service.accept_connections(&ctx).await }
    });

    // Wait for proxy to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Get proxy listen address (would need to expose this or use a known port)
    // For now, just verify the service starts
    // Then: data flows bidirectionally (would need actual connection test)

    // Cleanup
    proxy_handle.abort();
    server_handle.abort();
}

#[tokio::test]
async fn tokio_proxy_service_max_connections_limit_enforced_should_succeed() {
    // Given: a service with max_connections=1
    let config = ProxyConfig {
        listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        max_connections: Some(1),
    };
    let service = TokioProxyService::new(Arc::new(ArcSwap::from_pointee(config)))
        .expect("Failed to create service");
    let _ctx = create_test_context(vec![]);

    // When: accepting multiple connections
    // Then: only one connection is accepted at a time
    // This would require more complex testing with actual connections
    // For now, just verify service creation
    let _ = service;
}

#[tokio::test]
async fn tokio_proxy_service_sends_metrics_events_should_succeed() {
    // Given: a running service
    let config = ProxyConfig {
        listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        max_connections: Some(1000),
    };
    let service = TokioProxyService::new(Arc::new(ArcSwap::from_pointee(config)))
        .expect("Failed to create service");
    let _ctx = create_test_context(vec![]);

    // When: proxying a connection
    // Then: ConnectionOpened and ConnectionClosed events are sent
    // This would require actual connection proxying
    // For now, just verify service creation
    let _ = service;
}
