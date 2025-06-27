mod api;
mod auth;
mod config;
mod errors;
mod monitor;
mod recommend;

use crate::api::ApiClient;
use clap::Parser;
use std::time::Duration;
use sysinfo::System;
use tokio::time::Instant;
use uuid::Uuid;
use cli_table::{print_stdout, Table, WithTitle};

#[derive(Parser, Debug)]
#[clap(name = "vm-monitor", version = "0.1.0", author = "Farhan")]
#[clap(about = "Monitors VM resources and sends data to a remote API.")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Initialize the agent with API endpoint and instance name
    Init {
        #[clap(long, help = "Remote API base URL")]
        api_url: String,
        #[clap(long, help = "User-defined name for this VM instance")]
        name: String,
        #[clap(long, help = "Monitoring interval in seconds", default_value_t = 60)]
        interval: u64,
        #[clap(long, help = "Number of metrics to batch before sending", default_value_t = 10)]
        batch_size: usize,
    },
    /// Start monitoring and sending data (runs as a daemon-like foreground process)
    Start {
        #[clap(long, help = "Override monitoring interval in seconds from config")]
        interval: Option<u64>,
    },
    /// Show current system status and configuration
    Status,
    Recommend {
        #[clap(long, help = "Collect usage data for this many seconds before recommending", default_value_t = 60)]
        duration: u64,

        #[clap(long, help = "Optional: Filter recommendations by region (e.g., 'us-east', 'europe')")]
        region: Option<String>,
    },
}

async fn handle_init(
    api_url: String,
    instance_name: String,
    interval: u64,
    batch_size: usize,
) -> anyhow::Result<()> {
    log::info!(
        "Initializing new VmMonitor agent for instance: {}",
        instance_name
    );

    if config::load_config().is_ok() {
        log::warn!("Existing configuration found. Re-initializing will overwrite it.");
        // Add a prompt here in a real app: "Overwrite? [y/N]"
    }
    
    let instance_id = Uuid::new_v4();
    let api_key = auth::generate_api_key();
    log::info!("Generated Instance ID: {}", instance_id);
    log::debug!("Generated API Key: {}", api_key); // Log only in debug, not for user display of full key.

    log::info!("Detecting cloud provider...");
    let cloud_provider = config::detect_cloud_provider().await;
    log::info!("Detected cloud provider: {:?}", cloud_provider);

    let monitoring_settings = config::MonitoringSettings {
        interval_seconds: interval,
        batch_size,
    };

    let new_config = config::Configuration {
        instance_id,
        instance_name: instance_name.clone(),
        api_url: api_url.clone(),
        api_key: api_key.clone(),
        cloud_provider,
        monitoring_settings,
        initialized_at: chrono::Utc::now(),
    };

    // Attempt to register with the remote API
    // This requires the API to be available. For testing, this might be mocked.
    log::info!("Registering instance with API at {}...", api_url);
    let api_client = ApiClient::new(new_config.clone()); // Clone config for API client
    match api_client.register_instance().await {
        Ok(response) => {
            log::info!(
                "Instance registered successfully with API: {}",
                response.message
            );
        }
        Err(e) => {
            // Log full error for diagnostics, return user-friendly error
            log::error!("Failed to register instance with API: {:?}", e);
            return Err(anyhow::anyhow!(
                "Failed to register with remote API. Please check API URL and network. Error: {}", e
            ));
        }
    }

    let config_path = config::save_config(&new_config)?;
    log::info!("Configuration saved to: {}", config_path.display());

    println!("VmMonitor Agent initialized successfully!");
    println!("Instance ID: {}", instance_id);
    println!("Instance Name: {}", instance_name);
    println!("API URL: {}", api_url);
    println!("API Key: {}... (stored in config)", &api_key[..8.min(api_key.len())]); // Show only a prefix
    println!("Config file: {}", config_path.display());

    Ok(())
}

async fn handle_start(cli_interval: Option<u64>) -> anyhow::Result<()> {
    let config = config::load_config().map_err(|e| {
        anyhow::anyhow!("Failed to load configuration: {}. Please run 'init' first.", e)
    })?;
    
    let api_client = ApiClient::new(config.clone());

    let monitoring_interval_secs = cli_interval.unwrap_or(config.monitoring_settings.interval_seconds);
    let batch_size = config.monitoring_settings.batch_size;

    log::info!(
        "Starting VM Monitor Agent for instance ID: {}",
        config.instance_id
    );
    log::info!(
        "Monitoring interval: {}s, Batch size: {}",
        monitoring_interval_secs,
        batch_size
    );
    println!(
        "VM Monitor agent started. Interval: {}s, Batch Size: {}. Press Ctrl+C to stop.",
        monitoring_interval_secs,
        batch_size
    );


    let mut sys = System::new_all(); // Initialize sysinfo system
    let mut metrics_buffer: Vec<monitor::SystemMetrics> = Vec::new();
    let mut last_heartbeat_time = Instant::now();
    let heartbeat_interval = Duration::from_secs(5 * 60); // 5 minutes

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(monitoring_interval_secs)) => {
                log::debug!("Collecting metrics...");
                let current_metrics = monitor::collect_metrics(config.instance_id, &mut sys);
                metrics_buffer.push(current_metrics);
                log::info!("Collected metrics. Buffer size: {}", metrics_buffer.len());

                if metrics_buffer.len() >= batch_size {
                    log::info!("Batch limit reached ({} items). Sending metrics...", metrics_buffer.len());
                    match api_client.send_metrics_batch(&metrics_buffer).await {
                        Ok(_) => {
                            log::info!("Successfully sent batch of {} metrics.", metrics_buffer.len());
                            metrics_buffer.clear();
                        }
                        Err(e) => {
                            log::error!("Failed to send metrics batch: {}", e);
                            // Strategy for unsent metrics: For MVP, clear to avoid OOM.
                            // A more robust solution might involve a persistent queue or retry logic.
                            if metrics_buffer.len() > batch_size * 5 { // Avoid unbounded growth
                                log::warn!("Metrics buffer too large, clearing {} items to prevent OOM.", metrics_buffer.len());
                                metrics_buffer.clear();
                            }
                        }
                    }
                }

                // Heartbeat logic
                if last_heartbeat_time.elapsed() >= heartbeat_interval {
                    log::info!("Sending heartbeat...");
                    match api_client.send_heartbeat().await {
                        Ok(_) => {
                            log::info!("Heartbeat sent successfully.");
                            last_heartbeat_time = Instant::now(); // Reset timer only on success
                        }
                        Err(e) => {
                            log::error!("Failed to send heartbeat: {}", e);
                            // Don't reset timer, will retry next cycle implicitly (or specific retry logic)
                        }
                    }
                }
            }
            // Handle shutdown signal (Ctrl+C)
            result = tokio::signal::ctrl_c() => {
                match result {
                    Ok(()) => {
                        log::info!("Shutdown signal received.");
                    }
                    Err(e) => {
                        log::error!("Failed to listen for shutdown signal: {}", e);
                    }
                }
                
                if !metrics_buffer.is_empty() {
                    log::info!("Sending remaining {} metrics before shutdown...", metrics_buffer.len());
                    if let Err(e) = api_client.send_metrics_batch(&metrics_buffer).await {
                        log::error!("Failed to send final metrics batch: {}", e);
                    } else {
                        log::info!("Final metrics batch sent successfully.");
                    }
                }
                log::info!("VmMonitor agent shutting down.");
                break; // Exit loop
            }
        }
    }
    Ok(())
}

async fn handle_status() -> anyhow::Result<()> {
    println!("VM Monitor Agent Status:\n");

    match config::load_config() {
        Ok(config) => {
            println!("Configuration loaded:");
            println!("  Instance ID: {}", config.instance_id);
            println!("  Instance Name: {}", config.instance_name);
            println!("  API URL: {}", config.api_url);
            println!(
                "  API Key: {}... (masked)",
                &config.api_key[..8.min(config.api_key.len())]
            );
            println!("  Cloud Provider: {:?}", config.cloud_provider);
            println!(
                "  Monitoring Interval: {}s",
                config.monitoring_settings.interval_seconds
            );
            println!("  Batch Size: {}", config.monitoring_settings.batch_size);
            println!("  Initialized At: {}", config.initialized_at);
            
            // Check API connection status
            let api_client = ApiClient::new(config.clone());
            match api_client.check_api_status().await {
                Ok(_) => println!("\nAPI Connection Status: Connected"),
                Err(e) => println!("\nAPI Connection Status: Error - {}", e),
            }

        }
        Err(e) => {
            println!("Configuration not found or error loading: {}", e);
            println!("Please run 'vm-monitor init' to configure the agent.");
            return Ok(()); // Not an error for status command if not initialized
        }
    }

    println!("\nCurrent System Metrics (real-time snapshot):");
    let mut sys = System::new_all();
    // Use a dummy instance ID if config is not available, or get from config if it is.
    // For simplicity, if config fails, we might not have an instance_id for metrics.
    // However, collect_metrics requires one. Let's use a placeholder if no config.
    let instance_id_for_metrics = config::load_config().map(|c| c.instance_id).unwrap_or_else(|_| Uuid::nil());
    let metrics = monitor::collect_metrics(instance_id_for_metrics, &mut sys);
    
    // Pretty print metrics (abbreviated for brevity)
    println!("  Timestamp: {}", metrics.timestamp);
    println!("  CPU Usage: {:.2}% ({} cores)", metrics.cpu_metrics.usage_percent, metrics.cpu_metrics.core_count);
    // Could add per-core if verbose: println!("    Per-core: {:?}", metrics.cpu_metrics.per_core_usage);
    println!("  Memory: {:.2} GB / {:.2} GB used ({:.2} GB available)", 
        metrics.memory_metrics.used_memory as f64 / (1024.0 * 1024.0 * 1024.0),
        metrics.memory_metrics.total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
        metrics.memory_metrics.available_memory as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!("  Swap: {:.2} GB / {:.2} GB used",
        metrics.memory_metrics.used_swap as f64 / (1024.0 * 1024.0 * 1024.0),
        metrics.memory_metrics.total_swap as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    println!("  System Uptime: {} seconds", metrics.system_info.uptime);
    // Further details for disks and network can be added.
    // For brevity, just show count of disks/networks.
    println!("  Disks Found: {}", metrics.disk_metrics.len());
    println!("  Network Interfaces Found: {}", metrics.network_metrics.len());

    Ok(())
}

async fn handle_recommend(duration_secs: u64, region: Option<String>) -> anyhow::Result<()> {
    println!("Collecting system usage data for {} seconds. Please wait...", duration_secs);

    let mut sys = System::new_all();
    let mut cpu_usage_samples: Vec<f32> = Vec::new();
    let mut memory_usage_samples: Vec<u64> = Vec::new();

    let sleep_interval = Duration::from_secs(1);
    for _ in 0..duration_secs {
        sys.refresh_cpu_all();
        sys.refresh_memory();
        cpu_usage_samples.push(sys.global_cpu_usage());
        memory_usage_samples.push(sys.used_memory());
        tokio::time::sleep(sleep_interval).await;
    }

    let avg_cpu_usage = cpu_usage_samples.iter().sum::<f32>() / cpu_usage_samples.len() as f32;
    let avg_mem_used_bytes = memory_usage_samples.iter().sum::<u64>() / memory_usage_samples.len() as u64;
    let avg_mem_used_gb = avg_mem_used_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    
    let physical_cpu_cores = System::physical_core_count().unwrap_or_else(|| sys.cpus().len()) as u32;

    println!("\n--- Usage Analysis Complete ---");
    println!("Average CPU Usage: {:.2}%", avg_cpu_usage);
    println!("Average Memory Used: {:.2} GB", avg_mem_used_gb);
    println!("Physical CPU Cores on this machine: {}", physical_cpu_cores);
    println!("-----------------------------\n");

    println!("Loading VM instance dataset...");
    let dataset = match recommend::load_vm_dataset() {
        Ok(data) => data,
        Err(e) => return Err(anyhow::anyhow!("Failed to load VM dataset: {}", e)),
    };

    println!("Finding recommendations...");
    let recommendations = recommend::recommend_vms(
        &dataset,
        avg_cpu_usage,
        physical_cpu_cores,
        avg_mem_used_gb,
        region.as_deref(),
    );

    if recommendations.is_empty() {
        println!("No recommendations to display.");
        return Ok(());
    }

    #[derive(Table)]
    struct RecommendationRow {
        #[table(title = "Provider")]
        provider: String,
        #[table(title = "Instance Name")]
        instance_name: String,
        #[table(title = "Region")]
        region: String,
        #[table(title = "vCPUs")]
        vcpus: u32,
        #[table(title = "Memory (GB)")]
        memory_gb: f32,
        #[table(title = "Est. Hourly Cost ($)")]
        hourly_cost: String,
        #[table(title = "Efficiency Score")]
        score: String,
    }

    let table_data: Vec<RecommendationRow> = recommendations.iter().map(|rec| {
        RecommendationRow {
            provider: rec.instance.provider.clone(),
            instance_name: rec.instance.instance_name.clone(),
            region: rec.instance.region.clone(),
            vcpus: rec.instance.vcpus,
            memory_gb: rec.instance.memory_gb,
            hourly_cost: format!("{:.4}", rec.instance.hourly_cost), // Format cost
            score: format!("{:.6}", rec.cost_per_needed_resource), // Format score
        }
    }).collect();

    println!("Top VM Recommendations (lower score is better):");
    print_stdout(table_data.with_title())?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging: RUST_LOG=info vm-monitor ...
    // Default to `info` if RUST_LOG is not set.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { api_url, name, interval, batch_size } => {
            handle_init(api_url, name, interval, batch_size).await?
        }
        Commands::Start { interval } => handle_start(interval).await?,
        Commands::Status => handle_status().await?,
        Commands::Recommend { duration, region } => {
            handle_recommend(duration, region).await?
        }
    }

    Ok(())
}