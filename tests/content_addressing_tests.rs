//! Tests for Content Addressing with Hash Verification

use gun::{sea::work, Gun, WorkOptions};
use serde_json::json;

#[tokio::test]
async fn test_content_addressing_hash_match() {
    let gun = Gun::new();
    let data = json!({"name": "test", "value": 42});

    // Calculate hash of data
    let data_str = serde_json::to_string(&data).unwrap();
    let hash = work(
        data_str.as_bytes(),
        None,
        WorkOptions {
            name: Some("SHA-256".to_string()),
            encode: Some("base64".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Create soul with hash
    let soul = format!("#{}", hash);

    // Put data to hash soul - should succeed
    let result = gun.get(&soul).put(data.clone()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_content_addressing_hash_mismatch() {
    let gun = Gun::new();
    let data = json!({"name": "test", "value": 42});

    // Use wrong hash
    let wrong_hash = "wronghash123";
    let soul = format!("#{}", wrong_hash);

    // Put data to wrong hash soul - should fail
    let result = gun.get(&soul).put(data.clone()).await;

    // Should fail with hash mismatch error
    match result {
        Ok(_) => panic!("Expected error for hash mismatch"),
        Err(e) => {
            let err_str = e.to_string();
            assert!(err_str.contains("hash mismatch") || err_str.contains("InvalidData"));
        }
    }
}
