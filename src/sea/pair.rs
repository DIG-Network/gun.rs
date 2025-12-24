//! Key pair generation
//! Based on Gun.js sea/pair.js
//! Generates ECDSA (P-256) keys for signing and ECDH (P-256) keys for encryption

use super::KeyPair;
use super::SeaError;
use base64::{engine::general_purpose, Engine as _};
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{PublicKey as EcdhPublicKey, SecretKey};
use rand::rngs::OsRng;

/// Generate a new key pair
/// Creates both signing keys (ECDSA) and encryption keys (ECDH)
/// Matches Gun.js format: pub = "x.y", priv = base64 encoded scalar
pub async fn generate_pair() -> Result<KeyPair, SeaError> {
    // Generate ECDSA key pair for signing
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);

    // Export signing keys
    let priv_bytes = signing_key.to_bytes();
    let priv_key = general_purpose::STANDARD_NO_PAD.encode(priv_bytes.as_slice());

    // Get uncompressed public key point
    let pub_point = verifying_key.to_encoded_point(false);
    let pub_bytes = pub_point.as_bytes();

    // Extract x, y from uncompressed format (0x04 || x || y)
    if pub_bytes.len() != 65 || pub_bytes[0] != 0x04 {
        return Err(SeaError::Crypto("Invalid public key format".to_string()));
    }

    let x = &pub_bytes[1..33];
    let y = &pub_bytes[33..65];

    // Convert to base64 (matching Gun.js format: x.y)
    let pub_key = format!(
        "{}.{}",
        general_purpose::STANDARD_NO_PAD.encode(x),
        general_purpose::STANDARD_NO_PAD.encode(y)
    );

    // Generate ECDH key pair for encryption
    // Use SecretKey for persistent storage (EphemeralSecret doesn't expose raw bytes)
    let ecdh_secret = SecretKey::random(&mut OsRng);
    let ecdh_public = EcdhPublicKey::from_secret_scalar(&ecdh_secret.to_nonzero_scalar());

    // Export ECDH keys in same format
    let pub_point = ecdh_public.to_encoded_point(false);
    let epub_bytes = pub_point.as_bytes();

    if epub_bytes.len() != 65 || epub_bytes[0] != 0x04 {
        return Err(SeaError::Crypto(
            "Invalid ECDH public key format".to_string(),
        ));
    }

    let ex = &epub_bytes[1..33];
    let ey = &epub_bytes[33..65];

    let epub_key = Some(format!(
        "{}.{}",
        general_purpose::STANDARD_NO_PAD.encode(ex),
        general_purpose::STANDARD_NO_PAD.encode(ey)
    ));

    // Export ECDH private key (32 bytes for P-256)
    let epriv_bytes = ecdh_secret.to_bytes();
    let epriv_key = Some(general_purpose::STANDARD_NO_PAD.encode(epriv_bytes.as_slice()));

    Ok(KeyPair {
        pub_key,
        priv_key,
        epub_key,
        epriv_key,
    })
}
