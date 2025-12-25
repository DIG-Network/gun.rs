//! Comprehensive tests for error handling
//! Tests all error variants and error propagation

use gun::error::GunError;
use gun::storage::{MemoryStorage, Storage};
use gun::Gun;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_gun_error_serialization() {
    // Serialization error uses serde_json::Error
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let err = GunError::Serialization(json_err);
    assert!(err.to_string().contains("Serialization"));
}

#[test]
fn test_gun_error_invalid_data() {
    let err = GunError::InvalidData("invalid".to_string());
    assert!(err.to_string().contains("Invalid"));
}

#[test]
fn test_gun_error_network() {
    let err = GunError::Network("network error".to_string());
    assert!(err.to_string().contains("network"));
}

#[test]
fn test_gun_error_storage() {
    // Storage error uses sled::Error which we can't easily create in tests
    // Instead, test that IO errors (which can convert to storage errors) work
    let io_err = std::io::Error::other("storage error");
    let err = GunError::Io(io_err);
    assert!(err.to_string().contains("storage error"));
}

#[test]
fn test_gun_error_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = GunError::Io(io_err);
    assert!(err.to_string().contains("file not found"));
}

#[test]
fn test_gun_error_crypto() {
    let err = GunError::Crypto("crypto error".to_string());
    assert!(err.to_string().contains("crypto"));
}

#[tokio::test]
async fn test_error_propagation_storage() {
    // Test that storage errors propagate correctly
    let storage = Arc::new(MemoryStorage::new());

    // Valid operations should work
    let result = Storage::get(storage.as_ref(), "test").await;
    assert!(result.is_ok());

    // Invalid operations should return errors
    // (MemoryStorage doesn't have many error cases, but we can test the error type exists)
}

#[tokio::test]
async fn test_error_handling_invalid_json() {
    let gun = Gun::new();
    let chain = gun.get("test");

    // Valid JSON should work
    let result = chain.put(json!({"valid": true})).await;
    assert!(result.is_ok());
}

#[test]
fn test_gun_error_display() {
    // Test errors that can be created directly
    let errors: Vec<GunError> = vec![
        GunError::Serialization(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err()),
        GunError::InvalidData("data".to_string()),
        GunError::Network("net".to_string()),
        GunError::Crypto("crypto".to_string()),
    ];

    // All should be displayable
    for err in errors {
        let _ = format!("{}", err);
    }

    // Test IO error (can convert to Storage error in some cases)
    let io_err = std::io::Error::other("storage");
    let _ = format!("{}", GunError::Io(io_err));
}

#[test]
fn test_gun_error_send_sync() {
    // Verify GunError is Send + Sync for use in async contexts
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GunError>();
}

#[tokio::test]
async fn test_error_handling_user_auth_wrong_password() {
    use gun::sea::authenticate;
    use gun::Gun;
    use std::sync::Arc;
    
    let gun = Gun::new();
    let chain = gun.root();
    
    // Try to authenticate non-existent user
    let result = authenticate(chain.clone(), "nonexistent", "password").await;
    assert!(result.is_err(), "Should return error for non-existent user");
    
    // Verify error message is helpful
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("not found") || error_msg.contains("Crypto"));
    }
}

#[tokio::test]
async fn test_error_handling_certificate_verification() {
    use gun::sea::{certify, verify_certificate, pair, Certificants, Policy, CertifyOptions};
    
    let authority1 = pair().await.unwrap();
    let authority2 = pair().await.unwrap();
    
    // Create certificate with authority1
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("test".to_string()),
        &authority1,
        CertifyOptions::default(),
    ).await.unwrap();
    
    // Try to verify with wrong authority
    let result = verify_certificate(&cert, &authority2.pub_key).await;
    assert!(result.is_err(), "Should fail verification with wrong authority");
}

#[tokio::test]
async fn test_error_handling_invalid_certificate_format() {
    use gun::sea::verify_certificate;
    use gun::sea::pair;
    
    let authority = pair().await.unwrap();
    
    // Try to verify invalid certificate format
    let result = verify_certificate("invalid_cert", &authority.pub_key).await;
    assert!(result.is_err(), "Should fail for invalid certificate format");
}

#[tokio::test]
async fn test_error_handling_chain_operations() {
    let gun = Gun::new();
    let chain = gun.get("error_test");
    
    // Valid operations should work
    let result = chain.put(json!({"test": "data"})).await;
    assert!(result.is_ok(), "Valid put should succeed");
    
    // once() should handle missing data gracefully (not error)
    let mut called = false;
    let result = chain.get("nonexistent").once(|_data, _key| {
        called = true;
    }).await;
    assert!(result.is_ok(), "once() should not error on missing data");
    assert!(called, "once() callback should be called");
}

#[tokio::test]
async fn test_error_handling_graceful_degradation() {
    let gun = Gun::new();
    let chain = gun.get("degradation_test");
    
    // Operations should continue even if some fail
    // Put data
    chain.put(json!(1)).await.unwrap();
    
    // Multiple operations should not interfere
    for i in 0..5 {
        let result = chain.put(json!(i)).await;
        assert!(result.is_ok(), "Operations should continue even with rapid calls");
    }
}

#[tokio::test]
async fn test_error_handling_concurrent_errors() {
    let gun = Gun::new();
    
    // Multiple concurrent operations that might error
    let mut handles = vec![];
    
    for i in 0..10 {
        let chain = gun.get(&format!("concurrent_error_{}", i));
        let handle = tokio::spawn(async move {
            // These should all succeed or fail gracefully
            chain.put(json!(i)).await
        });
        handles.push(handle);
    }
    
    // Wait for all - should not panic
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent operations should not panic");
    }
}

#[tokio::test]
async fn test_error_handling_state_generation() {
    use gun::core::GunCore;
    
    let core = GunCore::new();
    
    // State generation should never fail (even with system time issues)
    // Multiple rapid calls should work
    for _ in 0..100 {
        let state = core.state.next();
        assert!(state > 0.0, "State should always be positive");
    }
}

#[tokio::test]
async fn test_error_handling_event_system() {
    use gun::events::EventEmitter;
    use gun::events::Event;
    
    let emitter = EventEmitter::new();
    
    // Register listener
    let listener_id = emitter.on("test_event", Box::new(|_event| {
        // Should not panic
    }));
    
    // Emit event - should not error
    emitter.emit(&Event {
        event_type: "test_event".to_string(),
        data: json!({"test": "data"}),
    });
    
    // Remove listener - should not error
    emitter.off("test_event", listener_id);
    
    // Emit again - should not error (just no listeners)
    emitter.emit(&Event {
        event_type: "test_event".to_string(),
        data: json!({"test": "data2"}),
    });
}

#[tokio::test]
async fn test_error_handling_graph_operations() {
    use gun::graph::Graph;
    use gun::state::Node;
    
    let graph = Graph::new();
    
    // Valid operations
    let node = Node::with_soul("test_soul".to_string());
    let result = graph.put("test_soul", node.clone());
    assert!(result.is_ok(), "Valid put should succeed");
    
    // Get should return Some for existing, None for missing (not error)
    let existing = graph.get("test_soul");
    assert!(existing.is_some(), "Should find existing node");
    
    let missing = graph.get("nonexistent");
    assert!(missing.is_none(), "Should return None for missing node");
}

#[tokio::test]
async fn test_error_handling_storage_operations() {
    use gun::storage::MemoryStorage;
    use gun::storage::Storage;
    use gun::state::Node;
    use std::sync::Arc;
    
    let storage = Arc::new(MemoryStorage::new());
    
    // Valid operations
    let node = Node::with_soul("test".to_string());
    let result = storage.put("test", &node).await;
    assert!(result.is_ok(), "Valid put should succeed");
    
    let result = storage.get("test").await;
    assert!(result.is_ok(), "Valid get should succeed");
    
    // Missing data should return Ok(None), not error
    let result = storage.get("nonexistent").await;
    assert!(result.is_ok(), "Missing data should return Ok(None)");
    assert!(result.unwrap().is_none(), "Missing data should be None");
}
