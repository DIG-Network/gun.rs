//! SEA (Security, Encryption, Authorization) module
//! Based on Gun.js sea/ directory
//! Provides encryption, authentication, and authorization capabilities

mod decrypt;
mod encrypt;
mod pair;
mod secret;
mod sign;
mod user;
mod verify;

pub use decrypt::*;
pub use encrypt::*;
pub use pair::*;
pub use secret::*;
pub use sign::*;
pub use user::*;
pub use verify::*;

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
pub async fn pair() -> Result<KeyPair, SeaError> {
    pair::generate_pair().await
}

/// User authentication structure
pub struct UserAuth {
    pub pair: KeyPair,
    pub alias: Option<String>,
}

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
