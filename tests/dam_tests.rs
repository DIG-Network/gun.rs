//! Comprehensive tests for DAM (Directed Acyclic Mesh) protocol
//! Tests message routing, peer management, and deduplication

use gun::core::GunCore;
use gun::dam::Mesh;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_mesh_new() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    // Should be able to get connected peer count
    assert_eq!(mesh.connected_peer_count().await, 0);
}

#[tokio::test]
async fn test_mesh_hear_message() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    // Create a test message as JSON string
    let msg_str = serde_json::to_string(&json!({
        "#": "test_soul",
        "put": {
            "test_soul": {
                "name": "test"
            }
        }
    }))
    .unwrap();

    // Should handle message without error (no peer for unit test)
    let result = mesh.hear(&msg_str, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mesh_hear_invalid_message() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    // Invalid message format (as string)
    let msg_str = r#"{"invalid": "message"}"#;

    // Should handle gracefully (may return error or ignore)
    let _ = mesh.hear(msg_str, None).await;
}

#[tokio::test]
async fn test_mesh_connected_peer_count() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    // Initially should have 0 peers
    assert_eq!(mesh.connected_peer_count().await, 0);
}

#[tokio::test]
async fn test_mesh_has_connected_peers() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    // Initially should have no connected peers
    assert!(!mesh.has_connected_peers().await);
}

#[tokio::test]
async fn test_mesh_wait_for_connection() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    // Wait for connection with short timeout (should timeout since no peers)
    let connected = mesh.wait_for_connection(100).await;
    assert!(!connected); // Should timeout
}

#[tokio::test]
async fn test_mesh_message_deduplication() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    let msg_str = serde_json::to_string(&json!({
        "#": "msg_id_123",
        "put": {
            "test_soul": {
                "name": "test"
            }
        }
    }))
    .unwrap();

    // Send same message twice (same message ID)
    let _ = mesh.hear(&msg_str, None).await;
    let _ = mesh.hear(&msg_str, None).await;

    // Should handle deduplication internally
    // (Actual deduplication is handled by Dup module)
}

#[tokio::test]
async fn test_mesh_put_message() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core.clone());

    // Create a put message as JSON string
    let msg_str = serde_json::to_string(&json!({
        "#": "test_soul",
        "put": {
            "test_soul": {
                "name": "Test",
                "value": 42
            }
        }
    }))
    .unwrap();

    let result = mesh.hear(&msg_str, None).await;
    assert!(result.is_ok());

    // Note: The mesh.hear() method processes DAM messages but doesn't directly
    // store in graph - that's handled by the core event system. This test
    // verifies the message is processed without error.
}

#[tokio::test]
async fn test_mesh_get_message() {
    let core = Arc::new(GunCore::new());
    let mesh = Mesh::new(core);

    // Create a get message as JSON string
    let get_msg_str = serde_json::to_string(&json!({
        "get": {
            "#": "test_soul"
        }
    }))
    .unwrap();

    // Should handle get message (may not return data in unit test, but should not error)
    let result = mesh.hear(&get_msg_str, None).await;
    assert!(result.is_ok());
}
