//! Observability Initialization
//!
//! Initializes OpenTelemetry tracing with console output and OTLP-ready architecture

use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_stdout::SpanExporter;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::resource::create_resource;

static INIT_TRACING: OnceLock<()> = OnceLock::new();

/// Initialize OpenTelemetry tracing with console output
///
/// This function should be called once at application startup (typically in the CLI or each service).
/// It sets up:
/// - OpenTelemetry SDK with console exporter (for now, ready for OTLP swap)
/// - Tracing subscriber with fmt layer for console output
/// - Environment-based log filtering via RUST_LOG
///
/// # Arguments
/// * `service_name` - The name of the service for resource identification
/// * `service_version` - The version of the service (e.g., "1.0.0")
///
/// # Returns
/// * `Ok(())` if initialization succeeded
/// * `Err(Box<dyn std::error::Error>)` if initialization failed
///
/// # Note
/// This function can be called multiple times, but only the first call will initialize the global
/// tracer provider and subscriber. Subsequent calls are ignored. Each service should call this
/// with its own service name and version to ensure proper resource identification.
pub fn init_tracing(
    service_name: &str,
    service_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    INIT_TRACING.get_or_init(|| {
        // Create OpenTelemetry resource with the first service's name/version
        // Note: The resource is set at the provider level, so all tracers share it.
        // Individual tracers can still be created with different service names.
        let resource = create_resource(service_name, service_version);

        // Create console span exporter (can be swapped for OTLP later)
        let exporter = SpanExporter::default();

        // Create tracer provider with batch processor
        let tracer_provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .build();

        // Set as global tracer provider
        global::set_tracer_provider(tracer_provider);

        // Create environment filter (defaults to "info" if RUST_LOG not set)
        let filter_layer = tracing_subscriber::filter::EnvFilter::builder()
            .with_default_directive(Level::INFO.into())
            .from_env_lossy();

        // Create fmt layer with JSON format (production-ready structured logging)
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .json();

        // Create OpenTelemetry layer to bridge tracing to OpenTelemetry
        // Use the service name from the first initialization
        let service_name_owned = service_name.to_string();
        let otel_layer = tracing_opentelemetry::layer()
            .with_tracer(global::tracer(service_name_owned));

        // Initialize tracing subscriber with all layers
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(otel_layer)
            .init();
    });

    let _tracer = global::tracer(service_name.to_string());

    Ok(())
}
