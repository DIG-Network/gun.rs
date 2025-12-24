//! User authentication
//! Based on Gun.js sea/user.js
//! User creation, authentication, and password management

use super::{KeyPair, UserAuth, SeaError};
use super::pair;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use pbkdf2::pbkdf2_hmac;

/// Create a new user with key pair
/// Generates a key pair and optional alias
pub async fn create_user(alias: Option<String>) -> Result<UserAuth, SeaError> {
    let pair = pair::generate_pair().await?;
    Ok(UserAuth {
        pair,
        alias,
    })
}

/// Authenticate user with password
/// This is a simplified implementation - full Gun.js user auth involves
/// storing encrypted keys and password hashes in the graph
/// 
/// For now, we'll implement a basic password hash verification
pub async fn authenticate(alias: &str, password: &str) -> Result<UserAuth, SeaError> {
    // TODO: Full implementation would:
    // 1. Look up user in graph by alias
    // 2. Verify password hash
    // 3. Decrypt user's private keys
    // 4. Return authenticated UserAuth
    
    // For now, this is a placeholder
    Err(SeaError::Crypto("Full authentication not yet implemented - requires graph storage integration".to_string()))
}

/// Hash a password using PBKDF2
/// Returns base64-encoded hash
pub fn hash_password(password: &str, salt: &[u8]) -> String {
    let mut hash = vec![0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100000, &mut hash);
    general_purpose::STANDARD_NO_PAD.encode(&hash)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, salt: &[u8], hash: &str) -> bool {
    let computed_hash = hash_password(password, salt);
    computed_hash == hash
}

/// Generate a random salt for password hashing
pub fn generate_salt() -> Vec<u8> {
    use rand::RngCore;
    let mut salt = vec![0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}
