use crate::errors::VmMonitorError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{PathBuf};
use uuid::Uuid;

#[cfg(all(unix, feature = "unix_perms"))]
use nix::sys::stat::{fchmod, Mode};
#[cfg(all(unix, feature = "unix_perms"))]
use std::os::unix::io::AsRawFd;


const CONFIG_FILE_NAME: &str = "vm-monitor.json";
const APP_NAME: &str = "vm-monitor";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CloudProvider {
    AWS,
    GCP,
    Azure,
    Unknown(String), // Store reason if known
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonitoringSettings {
    pub interval_seconds: u64,
    pub batch_size: usize,
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        MonitoringSettings {
            interval_seconds: 60,
            batch_size: 10,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Configuration {
    pub instance_id: Uuid,
    pub instance_name: String,
    pub api_url: String,
    pub api_key: String,
    pub cloud_provider: CloudProvider,
    pub monitoring_settings: MonitoringSettings,
    pub initialized_at: DateTime<Utc>,
}

fn get_config_path() -> Result<PathBuf, VmMonitorError> {
    dirs::config_dir()
        .ok_or_else(|| VmMonitorError::ConfigError("Could not find config directory".to_string()))
        .map(|path| path.join(APP_NAME).join(CONFIG_FILE_NAME))
}

pub fn save_config(config: &Configuration) -> Result<PathBuf, VmMonitorError> {
    let path = get_config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    
    #[cfg(all(unix, feature = "unix_perms"))]
    {
        fchmod(file.as_raw_fd(), Mode::S_IRUSR | Mode::S_IWUSR)?; // 600 permissions
        log::debug!("Set config file permissions to 600 (Unix).");
    }
    #[cfg(not(all(unix, feature = "unix_perms")))]
    {
        log::warn!("Unix permissions (600) for config file not set (not on Unix or feature disabled).");
    }


    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, config)?;
    writer.flush()?;
    Ok(path)
}

pub fn load_config() -> Result<Configuration, VmMonitorError> {
    let path = get_config_path()?;
    if !path.exists() {
        return Err(VmMonitorError::ConfigError(format!(
            "Configuration file not found at {}. Please run 'init' command.",
            path.display()
        )));
    }
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Configuration = serde_json::from_str(&contents)?;
    Ok(config)
}

// Basic cloud provider detection
pub async fn detect_cloud_provider() -> CloudProvider {
    // AWS: Check for /sys/hypervisor/uuid starting with "ec2"
    if let Ok(uuid_content) = std::fs::read_to_string("/sys/hypervisor/uuid") {
        if uuid_content.starts_with("ec2") {
            log::info!("AWS detected via /sys/hypervisor/uuid");
            return CloudProvider::AWS;
        }
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new()); // Fallback client if builder fails

    // GCP: Metadata server
    let gcp_url = "http://metadata.google.internal/computeMetadata/v1/?recursive=false&alt=text";
    match client.get(gcp_url).header("Metadata-Flavor", "Google").send().await {
        Ok(resp) if resp.status().is_success() => {
            log::info!("GCP detected via metadata server");
            return CloudProvider::GCP;
        }
        Err(e) => log::debug!("GCP metadata server check failed: {}", e),
        Ok(resp) => log::debug!("GCP metadata server check failed with status: {}", resp.status()),
    }

    // Azure: Metadata server
    let azure_url = "http://169.254.169.254/metadata/instance?api-version=2021-02-01";
    match client.get(azure_url).header("Metadata", "true").send().await {
        Ok(resp) if resp.status().is_success() => {
            log::info!("Azure detected via metadata server");
            return CloudProvider::Azure;
        }
        Err(e) => log::debug!("Azure metadata server check failed: {}", e),
        Ok(resp) => log::debug!("Azure metadata server check failed with status: {}", resp.status()),
    }
    
    log::info!("No specific cloud provider detected, defaulting to Unknown.");
    CloudProvider::Unknown("Not AWS, GCP, or Azure, or metadata services unreachable/unresponsive".to_string())
}