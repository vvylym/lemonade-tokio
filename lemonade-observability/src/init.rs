//! Observability Initialization
//!
//! Initializes OpenTelemetry tracing and metrics with OTLP export

use std::sync::OnceLock;

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{BatchSpanProcessor, Sampler, SdkTracerProvider};
use opentelemetry_stdout::SpanExporter as StdoutSpanExporter;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::resource::create_resource;

static INIT_TRACING: OnceLock<()> = OnceLock::new();
static INIT_METRICS: OnceLock<()> = OnceLock::new();

/// Initialize OpenTelemetry tracing with OTLP export
///
/// This function should be called once at application startup (typically in the CLI or each service).
/// It sets up:
/// - OpenTelemetry SDK with OTLP exporter (gRPC or HTTP) or console exporter as fallback
/// - Tracing subscriber with fmt layer for console output
/// - Environment-based log filtering via RUST_LOG
///
/// # Arguments
/// * `service_name` - The name of the service for resource identification
/// * `service_version` - The version of the service (e.g., "0.1.0")
/// * `service_instance_id` - Unique identifier for the service instance (e.g., "lemonade-worker-1")
/// * `otlp_endpoint` - Optional OTLP endpoint (defaults from env or console exporter if not set)
/// * `otlp_protocol` - Optional OTLP protocol: "grpc" or "http/protobuf" (defaults from env or "grpc")
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
    service_instance_id: &str,
    otlp_endpoint: Option<&str>,
    otlp_protocol: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    INIT_TRACING.get_or_init(|| {
        // Create OpenTelemetry resource with the first service's name/version/instance_id
        // Note: The resource is set at the provider level, so all tracers share it.
        // Individual tracers can still be created with different service names.
        let resource =
            create_resource(service_name, service_version, service_instance_id);

        // Create tracer provider with appropriate exporter based on OTLP config availability
        let tracer_provider = if let (Some(endpoint), Some(protocol)) = (otlp_endpoint, otlp_protocol) {
            // Note: We can't use tracing::info! here because tracing isn't initialized yet
            // Use eprintln! which will be captured by Docker logs
            eprintln!("[OTLP] Initializing OTLP exporter: endpoint={}, protocol={}", endpoint, protocol);
            // Use OTLP exporter
            match protocol {
                "grpc" => {
                    let exporter = opentelemetry_otlp::SpanExporterBuilder::default()
                        .with_tonic()
                        .with_endpoint(endpoint)
                        .build()
                        .expect("Failed to build OTLP gRPC span exporter");

                    eprintln!("[OTLP] OTLP gRPC exporter built successfully");

                    // Create batch span processor with explicit runtime configuration
                    // The runtime is automatically detected from the rt-tokio feature
                    let batch_processor = BatchSpanProcessor::builder(exporter)
                        .build();

                    eprintln!("[OTLP] Batch span processor created with AlwaysOn sampler");

                    SdkTracerProvider::builder()
                        .with_span_processor(batch_processor)
                        .with_sampler(Sampler::AlwaysOn)
                        .with_resource(resource)
                        .build()
                }
                "http" => {
                    let exporter = opentelemetry_otlp::SpanExporterBuilder::default()
                        .with_http()
                        .with_endpoint(endpoint)
                        .build()
                        .expect("Failed to build OTLP HTTP span exporter");

                    eprintln!("[OTLP] OTLP HTTP exporter built successfully");

                    // Create batch span processor with explicit runtime configuration
                    // The runtime is automatically detected from the rt-tokio feature
                    let batch_processor = BatchSpanProcessor::builder(exporter)
                        .build();

                    eprintln!("[OTLP] Batch span processor created with AlwaysOn sampler");

                    SdkTracerProvider::builder()
                        .with_span_processor(batch_processor)
                        .with_sampler(Sampler::AlwaysOn)
                        .with_resource(resource)
                        .build()
                }
                _ => {
                    eprintln!("[OTLP] Warning: Unsupported OTLP protocol: {}. Falling back to console exporter.", protocol);
                    let exporter = StdoutSpanExporter::default();
                    SdkTracerProvider::builder()
                        .with_batch_exporter(exporter)
                        .with_resource(resource)
                        .build()
                }
            }
        } else {
            eprintln!("[OTLP] No OTLP config provided (endpoint={:?}, protocol={:?}), using console exporter fallback", otlp_endpoint, otlp_protocol);
            // Fallback to console exporter when OTLP config is not available
            let exporter = StdoutSpanExporter::default();
            SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(resource)
                .build()
        };

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

/// Initialize OpenTelemetry metrics with OTLP export
///
/// This function should be called once at application startup to set up metrics collection.
/// It sets up:
/// - OpenTelemetry SDK with OTLP metrics exporter
/// - Metrics provider for instrumenting application metrics
///
/// # Arguments
/// * `service_name` - The name of the service for resource identification
/// * `service_version` - The version of the service (e.g., "0.1.0")
/// * `service_instance_id` - Unique identifier for the service instance
/// * `otlp_endpoint` - Optional OTLP endpoint (defaults from env or http://localhost:4317)
/// * `otlp_protocol` - Optional OTLP protocol: "grpc" or "http/protobuf" (defaults from env or "grpc")
///
/// # Returns
/// * `Ok(())` if initialization succeeded
/// * `Err(Box<dyn std::error::Error>)` if initialization failed
pub fn init_metrics(
    service_name: &str,
    service_version: &str,
    service_instance_id: &str,
    otlp_endpoint: Option<&str>,
    otlp_protocol: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    INIT_METRICS.get_or_init(|| {
        let resource = create_resource(service_name, service_version, service_instance_id);

        let meter_provider = if let (Some(endpoint), Some(protocol)) = (otlp_endpoint, otlp_protocol) {
            eprintln!("[OTLP Metrics] Initializing OTLP metrics exporter: endpoint={}, protocol={}", endpoint, protocol);
            match protocol {
                "grpc" => {
                    let exporter = opentelemetry_otlp::MetricExporterBuilder::new()
                        .with_tonic()
                        .with_endpoint(endpoint)
                        .build()
                        .expect("Failed to build OTLP gRPC metrics exporter");

                    eprintln!("[OTLP Metrics] OTLP gRPC metrics exporter built successfully");

                    let reader = PeriodicReader::builder(exporter)
                        .with_interval(std::time::Duration::from_secs(10))
                        .build();

                    SdkMeterProvider::builder()
                        .with_resource(resource)
                        .with_reader(reader)
                        .build()
                }
                "http" => {
                    let exporter = opentelemetry_otlp::MetricExporterBuilder::new()
                        .with_http()
                        .with_endpoint(endpoint)
                        .build()
                        .expect("Failed to build OTLP HTTP metrics exporter");

                    eprintln!("[OTLP Metrics] OTLP HTTP metrics exporter built successfully");

                    let reader = PeriodicReader::builder(exporter)
                        .with_interval(std::time::Duration::from_secs(10))
                        .build();

                    SdkMeterProvider::builder()
                        .with_resource(resource)
                        .with_reader(reader)
                        .build()
                }
                _ => {
                    eprintln!("[OTLP Metrics] Warning: Unsupported OTLP protocol: {}. Metrics will not be exported.", protocol);
                    // Create a no-op meter provider if protocol is unsupported
                    SdkMeterProvider::builder()
                        .with_resource(resource)
                        .build()
                }
            }
        } else {
            eprintln!("[OTLP Metrics] No OTLP config provided (endpoint={:?}, protocol={:?}), metrics will not be exported", otlp_endpoint, otlp_protocol);
            // Create a no-op meter provider if OTLP config is not available
            SdkMeterProvider::builder()
                .with_resource(resource)
                .build()
        };

        global::set_meter_provider(meter_provider);
        eprintln!("[OTLP Metrics] Metrics provider initialized successfully");
    });

    Ok(())
}
