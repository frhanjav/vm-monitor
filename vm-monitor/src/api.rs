use crate::auth;
use crate::config::Configuration;
use crate::errors::VmMonitorError;
use crate::monitor::SystemMetrics;
use chrono::Utc;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Placeholder for API response if needed, e.g. registration returns specific data
#[derive(Deserialize, Debug)]
pub struct RegistrationResponse {
    pub message: String,
    // Potentially other fields returned by the API upon registration
}

#[derive(Serialize)]
struct RegistrationPayload<'a> {
    instance_id: &'a str,
    instance_name: &'a str,
    cloud_provider: &'a str,
    agent_api_key: &'a str,
}

#[derive(Serialize)]
struct HeartbeatPayload<'a> {
    instance_id: &'a str,
}

pub struct ApiClient {
    http_client: Client,
    config: Configuration, // Store a copy or reference to the config
}

impl ApiClient {
    pub fn new(config: Configuration) -> Self {
        ApiClient {
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|e| {
                    log::warn!("Failed to build custom HTTP client: {}. Using default.", e);
                    Client::new()
                }),
            config,
        }
    }

    async fn send_request<T: Serialize, R: for<'de> Deserialize<'de> + 'static>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, VmMonitorError> {
        let url = format!("{}{}", self.config.api_url, path);
        let timestamp = Utc::now().timestamp();
        
        let body_str = match body {
            Some(b) => serde_json::to_string(b)?,
            None => "".to_string(),
        };

        let signature = auth::sign_request(
            &self.config.api_key,
            timestamp,
            method.as_str(),
            path,
            &body_str,
        )?;

        let mut request_builder = self.http_client.request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("X-Request-Timestamp", timestamp.to_string())
            .header("X-Request-Signature", signature)
            .header("X-Instance-Id", self.config.instance_id.to_string());

        if method != Method::GET && !body_str.is_empty() {
            request_builder = request_builder.header("Content-Type", "application/json").body(body_str);
        }
        
        log::debug!("Sending API request: {} {} to {}", method, path, url);

        let response = request_builder.send().await?;

        let status = response.status();
        let response_text = response.text().await?; // Read text for logging before trying to parse JSON

        if status.is_success() {
            if response_text.is_empty() && std::any::TypeId::of::<R>() == std::any::TypeId::of::<()>() {
                serde_json::from_str(&response_text)
                    .map_err(|e| VmMonitorError::JsonError(e))
            } else if response_text.is_empty() {
                 Err(VmMonitorError::ApiError(format!(
                    "API request to {} {} succeeded with status {} but returned an empty non-JSON response.",
                    method, path, status
                )))
            } else {
                serde_json::from_str(&response_text)
                    .map_err(|e| VmMonitorError::ApiError(format!(
                        "Failed to parse successful API response from {} {}: {}. Response body: {}", method, path, e, response_text
                    )))
            }
        } else {
            log::error!(
                "API request to {} {} failed with status {}: {}",
                method, path, status, response_text
            );
            Err(VmMonitorError::ApiError(format!(
                "API request failed: {} - {}",
                status, response_text
            )))
        }
    }

    pub async fn register_instance(&self) -> Result<RegistrationResponse, VmMonitorError> {
        // Convert CloudProvider enum to string for the payload
        let cloud_provider_str = match &self.config.cloud_provider {
            crate::config::CloudProvider::AWS => "AWS",
            crate::config::CloudProvider::GCP => "GCP",
            crate::config::CloudProvider::Azure => "Azure",
            crate::config::CloudProvider::Unknown(s) => s.as_str(),
        };

        let payload = RegistrationPayload {
            instance_id: &self.config.instance_id.to_string(),
            instance_name: &self.config.instance_name,
            cloud_provider: cloud_provider_str,
            agent_api_key: &self.config.api_key,
        };
        // Assuming API endpoint for registration is /register
        self.send_request(Method::POST, "/v1/agent/register", Some(&payload)).await
    }

    pub async fn send_metrics_batch(&self, metrics: &[SystemMetrics]) -> Result<(), VmMonitorError> {
        // API might expect a wrapper object like {"metrics": [...]}
        // For now, assume it accepts a direct array of SystemMetrics
        // Assuming API endpoint for metrics is /metrics
        // The type R for send_request needs to be specified. If no response body, use `()` and handle.
        // For now, let's make a dummy response struct for empty successful calls.

        #[derive(Serialize)]
        struct MetricsBatch<'a> {
            metrics: &'a [SystemMetrics],
        }
        
        let batch = MetricsBatch { metrics };
        
        #[derive(Deserialize)] 
        struct EmptyResponse {}

        let _: EmptyResponse = self.send_request(Method::POST, "/v1/agent/metrics", Some(&batch)).await?;
        Ok(())
    }

    pub async fn send_heartbeat(&self) -> Result<(), VmMonitorError> {
        let payload = HeartbeatPayload {
            instance_id: &self.config.instance_id.to_string(),
        };
        // Assuming API endpoint for heartbeat is /heartbeat
        #[derive(Deserialize)] struct EmptyResponse {}
        let _: EmptyResponse = self.send_request(Method::POST, "/v1/agent/heartbeat", Some(&payload)).await?;
        Ok(())
    }

    // A simple ping for status check
    pub async fn check_api_status(&self) -> Result<(), VmMonitorError> {
        #[derive(Deserialize)] struct PingResponse { _message: Option<String> } // Or more specific health check response
        // Assuming a GET endpoint like /health or /ping
        let _: PingResponse = self.send_request(Method::GET, "/v1/health", Option::<&()>::None).await?;
        Ok(())
    }
}