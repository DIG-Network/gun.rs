//! Error types for Gun.rs
//! 
//! All errors that can occur in Gun operations are represented by the `GunError` enum.
//! This module provides comprehensive error handling for the entire library.

use thiserror::Error;

/// Main error type for all Gun operations
/// 
/// All operations in Gun return `GunResult<T>` which is `Result<T, GunError>`.
/// This enum covers all possible error conditions that can occur.
/// 
/// # Error Variants
/// 
/// - `InvalidData(String)`: Data format is invalid or doesn't match expected structure
/// - `Storage(#[from] sled::Error)`: Storage operation failed (disk full, corruption, etc.)
/// - `Serialization(#[from] serde_json::Error)`: JSON serialization/deserialization failed
/// - `Network(String)`: Network operation failed (connection lost, timeout, etc.)
/// - `InvalidSoul(String)`: Soul (node ID) format is invalid
/// - `NodeNotFound`: Requested node doesn't exist in the graph
/// - `Io(#[from] std::io::Error)`: I/O operation failed (file read/write, etc.)
/// - `UrlParseError(#[from] url::ParseError)`: URL parsing failed (invalid peer URL)
/// - `WebRTC(String)`: WebRTC operation failed (connection, signaling, etc.)
/// - `Crypto(String)`: Cryptographic operation failed (encryption, signing, etc.)
/// 
/// # Error Handling
/// 
/// All errors implement `std::error::Error` and can be:
/// - Converted to strings with `to_string()` or `Display` trait
/// - Used with `?` operator for error propagation
/// - Matched with pattern matching
/// 
/// # Example
/// 
/// ```rust,no_run
/// use gun::{Gun, GunError};
/// use serde_json::json;
/// 
/// # async fn example() -> Result<(), GunError> {
/// let gun = Gun::new();
/// 
/// // Operations return GunResult
/// let result = gun.get("test").put(json!({"data": 42})).await;
/// 
/// match result {
///     Ok(chain) => println!("Success: {:?}", chain),
///     Err(GunError::Storage(e)) => eprintln!("Storage error: {}", e),
///     Err(GunError::Network(msg)) => eprintln!("Network error: {}", msg),
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Error, Debug)]
pub enum GunError {
    /// Invalid data format or structure
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Storage operation failed (from sled database)
    #[error("Storage error: {0}")]
    Storage(#[from] sled::Error),

    /// JSON serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Network operation failed (connection, timeout, etc.)
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid soul (node ID) format
    #[error("Invalid soul: {0}")]
    InvalidSoul(String),

    /// Requested node not found in graph
    #[error("Node not found")]
    NodeNotFound,

    /// I/O operation failed (file operations, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// URL parsing failed (invalid peer URL format)
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// WebRTC operation failed (connection, signaling, etc.)
    #[error("WebRTC error: {0}")]
    WebRTC(String),

    /// Cryptographic operation failed (encryption, signing, etc.)
    #[error("Crypto error: {0}")]
    Crypto(String),
}

/// Result type alias for Gun operations
/// 
/// All Gun operations return `GunResult<T>` which is `Result<T, GunError>`.
/// This provides consistent error handling across the entire library.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use gun::{Gun, GunResult};
/// use serde_json::json;
/// 
/// async fn put_data() -> GunResult<()> {
///     let gun = Gun::new();
///     gun.get("test").put(json!({"value": 42})).await?;
///     Ok(())
/// }
/// ```
pub type GunResult<T> = Result<T, GunError>;
