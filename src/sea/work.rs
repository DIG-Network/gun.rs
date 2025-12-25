//! Proof of Work / Content Hashing
//! Based on Gun.js sea/work.js
//! Provides PBKDF2 key derivation and SHA-256 hashing for proof-of-work and content addressing

use super::SeaError;
use base64::{engine::general_purpose, Engine as _};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::task;

/// Options for SEA.work()
#[derive(Clone, Debug)]
pub struct WorkOptions {
    /// Algorithm name: "PBKDF2" (default) or "SHA-256" (when name starts with "sha")
    pub name: Option<String>,
    /// Number of iterations for PBKDF2 (default: 100000)
    pub iterations: Option<u32>,
    /// Salt for PBKDF2 (auto-generated if not provided)
    pub salt: Option<Vec<u8>>,
    /// Hash algorithm for PBKDF2 (default: SHA-256)
    pub hash: Option<String>,
    /// Output length in bits (default: 512 bits = 64 bytes)
    pub length: Option<usize>,
    /// Output encoding (default: "base64")
    pub encode: Option<String>,
}

impl Default for WorkOptions {
    fn default() -> Self {
        Self {
            name: Some("PBKDF2".to_string()),
            iterations: Some(100_000),
            salt: None,
            hash: Some("SHA-256".to_string()),
            length: Some(512), // 64 bytes * 8 bits
            encode: Some("base64".to_string()),
        }
    }
}

/// Compute proof-of-work or content hash
/// Based on Gun.js SEA.work()
///
/// # Arguments
/// * `data` - Data to hash/derive (can be string or bytes)
/// * `salt_or_pair` - Optional salt (Vec<u8>) or KeyPair (for epub extraction)
/// * `opt` - Work options
///
/// # Returns
/// Base64-encoded hash or derived key
///
/// # Examples
/// ```rust
/// use gun::sea::{work, WorkOptions};
///
/// // SHA-256 hash for content addressing
/// let hash = work(b"hello world", None, WorkOptions {
///     name: Some("SHA-256".to_string()),
///     ..Default::default()
/// }).await?;
///
/// // PBKDF2 key derivation
/// let key = work(b"password", Some(vec![1,2,3,4,5,6,7,8,9]), WorkOptions::default()).await?;
/// ```
pub async fn work(
    data: &[u8],
    salt_or_pair: Option<Vec<u8>>,
    opt: WorkOptions,
) -> Result<String, SeaError> {
    let opt = Arc::new(opt);
    let data = data.to_vec();

    // Check if SHA-256 mode (when name starts with "sha")
    let name_lower = opt
        .name
        .as_ref()
        .map(|n| n.to_lowercase())
        .unwrap_or_else(|| "pbkdf2".to_string());

    if name_lower.starts_with("sha") {
        // SHA-256 hashing mode
        return task::spawn_blocking(move || {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hasher.finalize();

            let encoded = match opt.encode.as_deref().unwrap_or("base64") {
                "base64" => general_purpose::STANDARD_NO_PAD.encode(hash),
                "hex" => hex::encode(hash),
                _ => general_purpose::STANDARD_NO_PAD.encode(hash),
            };

            Ok(encoded)
        })
        .await
        .map_err(|e| SeaError::Crypto(format!("Task join error: {}", e)))?
        .map_err(|e: SeaError| e);
    }

    // PBKDF2 key derivation mode (default)
    let salt = if let Some(ref salt) = salt_or_pair {
        salt.clone()
    } else if let Some(ref opt_salt) = opt.salt {
        opt_salt.clone()
    } else {
        // Generate random 9-byte salt (matching Gun.js)
        let mut salt_bytes = vec![0u8; 9];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        salt_bytes
    };

    let iterations = opt.iterations.unwrap_or(100_000);
    let length_bits = opt.length.unwrap_or(512);
    let length_bytes = length_bits / 8;

    // Perform PBKDF2 in blocking task (CPU-intensive)
    let result = task::spawn_blocking(move || {
        let mut output = vec![0u8; length_bytes];
        pbkdf2_hmac::<Sha256>(&data, &salt, iterations, &mut output);
        output
    })
    .await
    .map_err(|e| SeaError::Crypto(format!("Task join error: {}", e)))?;

    // Encode result
    let encoded = match opt.encode.as_deref().unwrap_or("base64") {
        "base64" => general_purpose::STANDARD_NO_PAD.encode(&result),
        "hex" => hex::encode(&result),
        _ => general_purpose::STANDARD_NO_PAD.encode(&result),
    };

    Ok(encoded)
}

/// Convenience function: work with string data
pub async fn work_string(
    data: &str,
    salt_or_pair: Option<Vec<u8>>,
    opt: WorkOptions,
) -> Result<String, SeaError> {
    work(data.as_bytes(), salt_or_pair, opt).await
}

/// Convenience function: work with JSON-serializable data
pub async fn work_json<T: serde::Serialize>(
    data: &T,
    salt_or_pair: Option<Vec<u8>>,
    opt: WorkOptions,
) -> Result<String, SeaError> {
    let json_str = serde_json::to_string(data)
        .map_err(|e| SeaError::Crypto(format!("JSON serialization error: {}", e)))?;
    work(json_str.as_bytes(), salt_or_pair, opt).await
}
