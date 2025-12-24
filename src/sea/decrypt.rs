//! Decryption
//! Based on Gun.js sea/decrypt.js
//! AES-GCM decryption with ECDH key derivation

use super::{SeaError, KeyPair};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use serde_json::Value;
use base64::{Engine as _, engine::general_purpose};

/// Decrypt data using AES-GCM
/// Uses ECDH to derive decryption key
pub async fn decrypt(encrypted_data: &Value, pair: &KeyPair, their_epub: Option<&str>) -> Result<Value, SeaError> {
    // Parse encrypted data
    let ct_b64 = encrypted_data.get("ct")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SeaError::Decryption("Missing ciphertext".to_string()))?;
    let iv_b64 = encrypted_data.get("iv")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SeaError::Decryption("Missing IV".to_string()))?;
    let s_b64 = encrypted_data.get("s")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SeaError::Decryption("Missing salt".to_string()))?;
    
    // Decode from base64
    let ciphertext = general_purpose::STANDARD_NO_PAD.decode(ct_b64)
        .map_err(|_| SeaError::Decryption("Invalid ciphertext encoding".to_string()))?;
    let iv_bytes = general_purpose::STANDARD_NO_PAD.decode(iv_b64)
        .map_err(|_| SeaError::Decryption("Invalid IV encoding".to_string()))?;
    let salt_bytes = general_purpose::STANDARD_NO_PAD.decode(s_b64)
        .map_err(|_| SeaError::Decryption("Invalid salt encoding".to_string()))?;
    
    // Derive AES key (same as encryption)
    let aes_key = if let Some(their_pub) = their_epub {
        let our_epriv = pair.epriv_key.as_ref()
            .ok_or_else(|| SeaError::Decryption("Missing epriv key".to_string()))?;
        let our_epub = pair.epub_key.as_ref()
            .ok_or_else(|| SeaError::Decryption("Missing epub key".to_string()))?;
        
        let shared_secret = crate::sea::secret::derive_secret(their_pub, our_epriv, our_epub).await?;
        crate::sea::encrypt::derive_aes_key(&shared_secret, &salt_bytes).await?
    } else {
        let epriv = pair.epriv_key.as_ref()
            .ok_or_else(|| SeaError::Decryption("Missing epriv key".to_string()))?;
        crate::sea::encrypt::derive_aes_key(epriv, &salt_bytes).await?
    };
    
    // Create AES-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|e| SeaError::Decryption(format!("Failed to create cipher: {}", e)))?;
    
    // Create nonce from IV
    let nonce = Nonce::from_slice(&iv_bytes);
    
    // Decrypt
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| SeaError::Decryption(format!("Decryption failed: {}", e)))?;
    
    // Decode from UTF-8 and parse JSON
    let msg = String::from_utf8(plaintext)
        .map_err(|e| SeaError::Decryption(format!("Invalid UTF-8: {}", e)))?;
    
    serde_json::from_str(&msg)
        .map_err(|e| SeaError::Decryption(format!("Invalid JSON: {}", e)))
}
