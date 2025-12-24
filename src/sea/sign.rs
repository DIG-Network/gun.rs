//! Digital signatures
//! Based on Gun.js sea/sign.js
//! ECDSA P-256 signing

use super::{KeyPair, SeaError};
use base64::{engine::general_purpose, Engine as _};
use p256::ecdsa::Signature;
use p256::ecdsa::{signature::Signer, SigningKey};
use serde_json::Value;
use sha2::Sha256;

/// Sign data with a key pair
/// Returns signed data in format: {m: message, s: signature}
///
/// The message is JSON-serialized and signed using ECDSA P-256
pub async fn sign(data: &Value, key_pair: &KeyPair) -> Result<Value, SeaError> {
    // Serialize data
    let message = serde_json::to_string(data)
        .map_err(|e| SeaError::Crypto(format!("Serialization error: {}", e)))?;

    // Create signing key from private key
    let priv_bytes = general_purpose::STANDARD_NO_PAD
        .decode(&key_pair.priv_key)
        .map_err(|_| SeaError::InvalidKey)?;

    // Convert Vec<u8> to fixed-size array for SigningKey
    if priv_bytes.len() != 32 {
        return Err(SeaError::InvalidKey);
    }
    let mut priv_array = [0u8; 32];
    priv_array.copy_from_slice(&priv_bytes);
    let signing_key = SigningKey::from_bytes(&priv_array.into())
        .map_err(|e| SeaError::Crypto(format!("Invalid private key: {}", e)))?;

    // Sign the message (ECDSA with SHA-256)
    let signature: Signature = signing_key.sign(message.as_bytes());
    let sig_bytes = signature.to_bytes();
    let sig_b64 = general_purpose::STANDARD_NO_PAD.encode(sig_bytes.as_slice());

    // Return in Gun.js format: {m: message, s: signature}
    Ok(serde_json::json!({
        "m": message,
        "s": sig_b64
    }))
}
