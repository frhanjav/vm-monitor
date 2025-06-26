use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum VmMonitorError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Failed to read system metadata for cloud detection: {0}")]
    CloudDetectionError(String),
    #[error("Filesystem error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("API communication error: {0}")]
    ApiError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Invalid input: {0}")]
    InputError(String),
    #[error("Monitoring error: {0}")]
    MonitorError(String),
}