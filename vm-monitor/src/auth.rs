use crate::errors::VmMonitorError;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_api_key() -> String {
    let mut key_bytes = [0u8; 32]; // 256 bits
    rand::thread_rng().fill_bytes(&mut key_bytes);
    STANDARD.encode(key_bytes) // Use the STANDARD engine to encode
}

pub fn sign_request(
    api_secret_key: &str,
    timestamp: i64,
    http_method: &str,
    request_path: &str,
    request_body: &str, // JSON string of the body
) -> Result<String, VmMonitorError> {
    let message = format!(
        "{}\n{}\n{}\n{}",
        timestamp,
        http_method.to_uppercase(),
        request_path,
        request_body
    );

    let mut mac = HmacSha256::new_from_slice(api_secret_key.as_bytes())
        .map_err(|e| VmMonitorError::AuthError(format!("Failed to initialize HMAC: {}", e)))?;
    mac.update(message.as_bytes());
    let signature_bytes = mac.finalize().into_bytes();
    Ok(STANDARD.encode(signature_bytes)) // Use the STANDARD engine to encode
}