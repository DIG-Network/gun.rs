//! Encryption
//! Based on Gun.js sea/encrypt.js
//! AES-GCM encryption with ECDH key derivation

use super::{KeyPair, SeaError};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use serde_json::Value;

/// Encrypt data using AES-GCM
/// Uses ECDH to derive encryption key from the recipient's public key
///
/// Format: {ct: ciphertext, iv: nonce, s: salt} (all base64)
pub async fn encrypt(
    data: &Value,
    pair: &KeyPair,
    their_epub: Option<&str>,
) -> Result<Value, SeaError> {
    // Serialize data to string
    let msg = serde_json::to_string(data)
        .map_err(|e| SeaError::Encryption(format!("Serialization error: {}", e)))?;

    // Generate random salt and IV (nonce)
    let mut salt_bytes = [0u8; 9];
    let mut iv_bytes = [0u8; 15];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    rand::thread_rng().fill_bytes(&mut iv_bytes);

    // Derive AES key
    // If their_epub is provided, use ECDH; otherwise use epriv directly (for self-encryption)
    let aes_key = if let Some(their_pub) = their_epub {
        // Use ECDH to derive shared secret
        let our_epriv = pair
            .epriv_key
            .as_ref()
            .ok_or_else(|| SeaError::Encryption("Missing epriv key".to_string()))?;
        let our_epub = pair
            .epub_key
            .as_ref()
            .ok_or_else(|| SeaError::Encryption("Missing epub key".to_string()))?;

        // Derive shared secret
        let shared_secret =
            crate::sea::secret::derive_secret(their_pub, our_epriv, our_epub).await?;

        // Combine shared secret with salt to create AES key
        derive_aes_key(&shared_secret, &salt_bytes).await?
    } else {
        // Use epriv directly for self-encryption (simplified)
        let epriv = pair
            .epriv_key
            .as_ref()
            .ok_or_else(|| SeaError::Encryption("Missing epriv key".to_string()))?;
        derive_aes_key(epriv, &salt_bytes).await?
    };

    // Create AES-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| SeaError::Encryption(format!("Failed to create cipher: {}", e)))?;

    // Create nonce from IV
    #[allow(deprecated)] // generic_array::from_slice is deprecated but aes-gcm still uses it
    let nonce = Nonce::from_slice(&iv_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, msg.as_bytes())
        .map_err(|e| SeaError::Encryption(format!("Encryption failed: {}", e)))?;

    // Encode everything as base64
    let ct_b64 = general_purpose::STANDARD_NO_PAD.encode(ciphertext);
    let iv_b64 = general_purpose::STANDARD_NO_PAD.encode(iv_bytes);
    let s_b64 = general_purpose::STANDARD_NO_PAD.encode(salt_bytes);

    // Return in Gun.js format
    Ok(serde_json::json!({
        "ct": ct_b64,
        "iv": iv_b64,
        "s": s_b64
    }))
}

/// Derive AES key from secret and salt
pub(crate) async fn derive_aes_key(secret: &str, salt: &[u8]) -> Result<Vec<u8>, SeaError> {
    // Use PBKDF2 to derive key (matching Gun.js behavior)
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    let secret_bytes = general_purpose::STANDARD_NO_PAD
        .decode(secret)
        .unwrap_or_else(|_| secret.as_bytes().to_vec());

    let mut key = vec![0u8; 32]; // AES-256 key size
    pbkdf2_hmac::<Sha256>(&secret_bytes, salt, 100000, &mut key);

    Ok(key)
}
