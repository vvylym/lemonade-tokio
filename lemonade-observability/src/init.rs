//! Observability Initialization
//!
//! Initializes OpenTelemetry tracing with console output and OTLP-ready architecture

use opentelemetry::global;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_stdout::SpanExporter;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::{Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::resource::create_resource;

static INIT_ONCE: Once = Once::new();

/// Initialize OpenTelemetry tracing with console output
///
/// This function should be called once at application startup (typically in the CLI).
/// It sets up:
/// - OpenTelemetry SDK with console exporter (for now, ready for OTLP swap)
/// - Tracing subscriber with fmt layer for console output
/// - Environment-based log filtering via RUST_LOG
///
/// # Arguments
/// * `service_name` - The name of the service for resource identification
///
/// # Returns
/// * `Ok(())` if initialization succeeded
/// * `Err(Box<dyn std::error::Error>)` if initialization failed
///
/// # Panics
/// This function will panic if called multiple times (use Once to prevent this)
pub fn init_tracing(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    INIT_ONCE.call_once(|| {
        // Create OpenTelemetry resource
        let resource = create_resource(service_name, None, None);

        // Create console span exporter (can be swapped for OTLP later)
        let exporter = SpanExporter::default();

        // Create tracer provider with batch processor
        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .build();

        // Set as global tracer provider
        global::set_tracer_provider(tracer_provider);

        // Determine log format from environment
        let log_format = std::env::var("OTEL_LOG_FORMAT")
            .unwrap_or_else(|_| "compact".to_string())
            .to_lowercase();

        // Create environment filter (defaults to "info" if RUST_LOG not set)
        let filter_layer = tracing_subscriber::filter::EnvFilter::builder()
            .with_default_directive(Level::INFO.into())
            .from_env_lossy();

        // Create fmt layer based on format preference
        let fmt_layer = if log_format == "json" {
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .json()
                .boxed()
        } else {
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_ansi(true)
                .compact()
                .boxed()
        };

        // Create OpenTelemetry layer to bridge tracing to OpenTelemetry
        let service_name_owned = service_name.to_string();
        let otel_layer = tracing_opentelemetry::layer()
            .with_tracer(global::tracer(service_name_owned))
            .boxed();

        // Initialize tracing subscriber with all layers
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(otel_layer)
            .init();
    });

    Ok(())
}
