//! Command handlers
//!
use lemonade_service::config::{Config, ConfigBuilder, WorkerAddress};
use std::{path::PathBuf, time::Duration};

/// Run a worker server
#[tracing::instrument(skip_all, fields(service.name = %framework, service.instance.id = ?name))]
pub async fn run_worker(
    framework: String,
    config_file: Option<PathBuf>,
    address: Option<String>,
    name: Option<String>,
    delay: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = if let Some(path) = config_file {
        // Load from file
        ConfigBuilder::from_file(Some(path))?
    } else if address.is_some() || name.is_some() || delay.is_some() {
        // Build from individual arguments (with fallback to env for missing values)
        let listen_address = if let Some(addr) = address {
            WorkerAddress::parse(&addr)?
        } else {
            // Try from env or use default
            ConfigBuilder::from_env()?.listen_address().clone()
        };

        let service_name = name.unwrap_or_else(|| {
            std::env::var("LEMONADE_WORKER_SERVICE_NAME")
                .unwrap_or_else(|_| "lemonade-worker".to_string())
        });

        let work_delay = if let Some(d) = delay {
            Duration::from_millis(d)
        } else {
            std::env::var("LEMONADE_WORKER_WORK_DELAY_MS")
                .unwrap_or_else(|_| "20".to_string())
                .parse::<u64>()
                .map(Duration::from_millis)
                .unwrap_or(Duration::from_millis(20))
        };

        Config::new(listen_address, service_name, work_delay)
    } else {
        // No arguments provided, load from env
        ConfigBuilder::from_env()?
    };

    let framework_lower = framework.to_lowercase();
    match framework_lower.as_str() {
        "actix" | "actix-web" => {
            lemonade_worker_actix::run(config).await?;
        }
        "axum" => {
            lemonade_worker_axum::run(config).await?;
        }
        "hyper" => {
            lemonade_worker_hyper::run(config).await?;
        }
        "rocket" => {
            lemonade_worker_rocket::run(config).await?;
        }
        _ => {
            return Err(format!(
                "Unknown framework: {}. Supported: actix, axum, hyper, rocket",
                framework
            )
            .into());
        }
    }

    Ok(())
}

/// Run a load balancer
pub async fn run_load_balancer(
    config_file: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    lemonade_load_balancer::run(config_file).await
}
