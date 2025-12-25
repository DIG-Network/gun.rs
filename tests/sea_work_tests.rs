//! Tests for SEA.work() - Proof of Work / Content Hashing

use gun::sea::{work, WorkOptions};

#[tokio::test]
async fn test_work_sha256() {
    let data = b"hello world";
    let result = work(
        data,
        None,
        WorkOptions {
            name: Some("SHA-256".to_string()),
            encode: Some("base64".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // SHA-256 of "hello world" in base64
    // Should be consistent
    // 32 bytes = 43 base64 characters (no padding with STANDARD_NO_PAD)
    assert!(!result.is_empty());
    assert_eq!(result.len(), 43); // Base64 length for 32 bytes (no padding)
}

#[tokio::test]
async fn test_work_pbkdf2() {
    let data = b"password";
    let salt = b"saltsalt";
    let result = work(
        data,
        Some(salt.to_vec()),
        WorkOptions {
            name: Some("PBKDF2".to_string()),
            iterations: Some(1000), // Lower for testing
            encode: Some("base64".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_work_hex_encoding() {
    let data = b"test";
    let result = work(
        data,
        None,
        WorkOptions {
            name: Some("SHA-256".to_string()),
            encode: Some("hex".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Hex encoding should be 64 characters (32 bytes * 2)
    assert_eq!(result.len(), 64);
}

#[tokio::test]
async fn test_work_auto_salt() {
    let data = b"data";
    // No salt provided, should auto-generate
    let result1 = work(data, None, WorkOptions::default()).await.unwrap();
    let result2 = work(data, None, WorkOptions::default()).await.unwrap();

    // Results should be different due to random salt
    assert_ne!(result1, result2);
}

#[tokio::test]
async fn test_work_deterministic_with_salt() {
    let data = b"data";
    let salt = b"fixedsalt";

    let result1 = work(data, Some(salt.to_vec()), WorkOptions::default())
        .await
        .unwrap();

    let result2 = work(data, Some(salt.to_vec()), WorkOptions::default())
        .await
        .unwrap();

    // Same salt should produce same result
    assert_eq!(result1, result2);
}
