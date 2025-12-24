//! Signature verification
//! Based on Gun.js sea/verify.js
//! ECDSA P-256 verification

use super::SeaError;
use base64::{engine::general_purpose, Engine as _};
use p256::ecdsa::Signature;
use p256::ecdsa::{signature::Verifier, VerifyingKey};
use serde_json::Value;

/// Verify a signature
/// Returns the verified message data if valid
///
/// Takes signed data in format {m: message, s: signature} and a public key
pub async fn verify(signed_data: &Value, pub_key: &str) -> Result<Value, SeaError> {
    let message = signed_data
        .get("m")
        .and_then(|v| v.as_str())
        .ok_or(SeaError::VerificationFailed)?;

    let signature = signed_data
        .get("s")
        .and_then(|v| v.as_str())
        .ok_or(SeaError::VerificationFailed)?;

    // Parse public key (format: x.y)
    let parts: Vec<&str> = pub_key.split('.').collect();
    if parts.len() != 2 {
        return Err(SeaError::InvalidKey);
    }

    let x = general_purpose::STANDARD_NO_PAD
        .decode(parts[0])
        .map_err(|_| SeaError::InvalidKey)?;
    let y = general_purpose::STANDARD_NO_PAD
        .decode(parts[1])
        .map_err(|_| SeaError::InvalidKey)?;

    // Reconstruct public key bytes (uncompressed format: 0x04 || x || y)
    let mut pub_bytes = vec![0x04u8];
    pub_bytes.extend_from_slice(&x);
    pub_bytes.extend_from_slice(&y);

    let verifying_key = VerifyingKey::from_sec1_bytes(&pub_bytes)
        .map_err(|e| SeaError::Crypto(format!("Invalid public key: {}", e)))?;

    // Decode signature (ECDSA signatures are 64 bytes: r || s)
    let sig_bytes = general_purpose::STANDARD_NO_PAD
        .decode(signature)
        .map_err(|_| SeaError::VerificationFailed)?;
    if sig_bytes.len() != 64 {
        return Err(SeaError::VerificationFailed);
    }
    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);
    let signature =
        Signature::from_bytes(&sig_array.into()).map_err(|_| SeaError::VerificationFailed)?;

    // Verify signature
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| SeaError::VerificationFailed)?;

    // Parse and return the message
    serde_json::from_str(message).map_err(|e| SeaError::Crypto(format!("Invalid JSON: {}", e)))
}
