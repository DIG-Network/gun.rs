//! SEA (Security, Encryption, Authorization) module
//! Based on Gun.js sea/ directory
//! Provides encryption, authentication, and authorization capabilities

mod certify;
mod decrypt;
mod encrypt;
mod pair;
mod secret;
mod sign;
mod user;
mod verify;
mod work;

pub use certify::*;
pub use decrypt::*;
pub use encrypt::*;
pub use pair::*;
pub use secret::*;
pub use sign::*;
pub use user::*;
pub use verify::*;
pub use work::*;

/// Key pair for signing and encryption
#[derive(Clone, Debug)]
pub struct KeyPair {
    /// Public key for signing (ECDSA, P-256)
    pub pub_key: String,
    /// Private key for signing (ECDSA, P-256)
    pub priv_key: String,
    /// Public key for encryption (ECDH, P-256)
    pub epub_key: Option<String>,
    /// Private key for encryption (ECDH, P-256)
    pub epriv_key: Option<String>,
}

/// Generate a new key pair for signing and encryption
/// Based on Gun.js SEA.pair()
/// 
/// # Returns
/// A `KeyPair` containing:
/// - `pub_key`: Public key for signing (ECDSA P-256) in base64 x.y format
/// - `priv_key`: Private key for signing (ECDSA P-256) in base64 format
/// - `epub_key`: Public key for encryption (ECDH P-256) in base64 x.y format
/// - `epriv_key`: Private key for encryption (ECDH P-256) in base64 format
/// 
/// # Errors
/// Returns `SeaError::Crypto` if key generation fails
/// 
/// # Example
/// ```rust,no_run
/// use gun::sea::pair;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let keypair = pair().await?;
/// println!("Public key: {}", keypair.pub_key);
/// # Ok(())
/// # }
/// ```
pub async fn pair() -> Result<KeyPair, SeaError> {
    pair::generate_pair().await
}

/// User authentication structure
/// 
/// Contains the authenticated user's key pair and optional alias.
/// This is returned by `create_user()` and `authenticate()` functions.
/// 
/// # Fields
/// - `pair`: The user's key pair (public and private keys)
/// - `alias`: Optional user alias/username
pub struct UserAuth {
    pub pair: KeyPair,
    pub alias: Option<String>,
}

/// SEA module error types
/// 
/// All errors that can occur in SEA (Security, Encryption, Authorization) operations.
/// 
/// # Variants
/// - `Crypto(String)`: General cryptographic error with message
/// - `InvalidKey`: Key format is invalid or cannot be parsed
/// - `VerificationFailed`: Signature verification failed (data may be tampered or wrong key)
/// - `Encryption(String)`: Error during encryption operation
/// - `Decryption(String)`: Error during decryption operation
/// 
/// # Example
/// ```rust,no_run
/// use gun::sea::{pair, sign, verify, SeaError};
/// use serde_json::json;
/// 
/// # async fn example() -> Result<(), SeaError> {
/// let keypair = pair().await?;
/// let data = json!({"message": "hello"});
/// let signed = sign(&data, &keypair).await?;
/// 
/// // Verification should succeed
/// let verified = verify(&signed, &keypair.pub_key).await?;
/// assert_eq!(verified, data);
/// 
/// // Wrong key should fail
/// let wrong_keypair = pair().await?;
/// let result = verify(&signed, &wrong_keypair.pub_key).await;
/// assert!(matches!(result, Err(SeaError::VerificationFailed)));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum SeaError {
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Invalid key format")]
    InvalidKey,
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Decryption error: {0}")]
    Decryption(String),
}
