//! ECDH key derivation (shared secret)
//! Based on Gun.js sea/secret.js
//! Derives a shared secret from ECDH key exchange

use super::SeaError;
use p256::{SecretKey, PublicKey, ecdh::SharedSecret};
use base64::{Engine as _, engine::general_purpose};

/// Derive shared secret from ECDH key exchange
/// Takes a public key (epub) and a key pair with epriv/epub
/// Returns the derived secret key (base64 encoded)
pub async fn derive_secret(
    their_epub: &str,
    our_epriv: &str,
    our_epub: &str,
) -> Result<String, SeaError> {
    // Parse their public key (format: x.y)
    let their_pub = parse_epub(their_epub)?;
    
    // Parse our keys
    let our_pub = parse_epub(our_epub)?;
    let our_priv_bytes = general_purpose::STANDARD_NO_PAD.decode(our_epriv)
        .map_err(|_| SeaError::InvalidKey)?;
    
    // Create our secret key from private key bytes (32 bytes for P-256)
    if our_priv_bytes.len() != 32 {
        return Err(SeaError::InvalidKey);
    }
    let mut priv_array = [0u8; 32];
    priv_array.copy_from_slice(&our_priv_bytes);
    let our_secret = SecretKey::from_bytes(&priv_array.into())
        .map_err(|_| SeaError::InvalidKey)?;
    
    // Derive shared secret using ECDH
    // Use the ecdh module's diffie_hellman function
    let shared_secret = p256::ecdh::diffie_hellman(
        our_secret.to_nonzero_scalar(),
        their_pub.as_affine()
    );
    
    // Extract the x-coordinate of the shared point as the secret
    // This is what Gun.js does - uses the x coordinate
    let shared_point = shared_secret.raw_secret_bytes();
    
    // Return as base64 (matching Gun.js format)
    Ok(general_purpose::STANDARD_NO_PAD.encode(&shared_point.as_slice()))
}

/// Parse an epub key (format: x.y base64) into a PublicKey
fn parse_epub(epub: &str) -> Result<PublicKey, SeaError> {
    let parts: Vec<&str> = epub.split('.').collect();
    if parts.len() != 2 {
        return Err(SeaError::InvalidKey);
    }
    
    let x = general_purpose::STANDARD_NO_PAD.decode(parts[0])
        .map_err(|_| SeaError::InvalidKey)?;
    let y = general_purpose::STANDARD_NO_PAD.decode(parts[1])
        .map_err(|_| SeaError::InvalidKey)?;
    
    // Reconstruct uncompressed public key (0x04 || x || y)
    let mut pub_bytes = vec![0x04u8];
    pub_bytes.extend_from_slice(&x);
    pub_bytes.extend_from_slice(&y);
    
    // Import as public key
    PublicKey::from_sec1_bytes(&pub_bytes)
        .map_err(|_| SeaError::InvalidKey)
}

