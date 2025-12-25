//! Comprehensive tests for all storage backends
//! Tests MemoryStorage, SledStorage, and LocalStorage

use gun::state::Node;
use gun::storage::{LocalStorage, MemoryStorage, SledStorage, Storage};
use serde_json::json;

// Helper to create a test node
fn create_test_node(soul: &str, key: &str, value: &serde_json::Value) -> Node {
    let mut node = Node::with_soul(soul.to_string());
    node.data.insert(key.to_string(), value.clone());
    node
}

// ========== MemoryStorage Tests ==========

#[tokio::test]
async fn test_memory_storage_new() {
    let storage = MemoryStorage::new();
    // Should be empty initially
    let result = storage.has("test").await.unwrap();
    assert!(!result);
}

#[tokio::test]
async fn test_memory_storage_put_get() {
    let storage = MemoryStorage::new();
    let node = create_test_node("soul1", "name", &json!("Alice"));

    // Put node
    storage.put("soul1", &node).await.unwrap();

    // Get node
    let retrieved = storage.get("soul1").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_node = retrieved.unwrap();
    assert_eq!(retrieved_node.get_soul(), Some("soul1".to_string()));
    assert_eq!(retrieved_node.data.get("name"), Some(&json!("Alice")));
}

#[tokio::test]
async fn test_memory_storage_get_non_existent() {
    let storage = MemoryStorage::new();
    let result = storage.get("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_memory_storage_has() {
    let storage = MemoryStorage::new();
    let node = create_test_node("soul1", "name", &json!("Alice"));

    assert!(!storage.has("soul1").await.unwrap());
    storage.put("soul1", &node).await.unwrap();
    assert!(storage.has("soul1").await.unwrap());
}

#[tokio::test]
async fn test_memory_storage_update() {
    let storage = MemoryStorage::new();
    let node1 = create_test_node("soul1", "name", &json!("Alice"));
    let mut node2 = create_test_node("soul1", "name", &json!("Bob"));
    node2.data.insert("age".to_string(), json!(30));

    storage.put("soul1", &node1).await.unwrap();
    storage.put("soul1", &node2).await.unwrap();

    let retrieved = storage.get("soul1").await.unwrap().unwrap();
    assert_eq!(retrieved.data.get("name"), Some(&json!("Bob")));
    assert_eq!(retrieved.data.get("age"), Some(&json!(30)));
}

#[tokio::test]
async fn test_memory_storage_multiple_nodes() {
    let storage = MemoryStorage::new();

    for i in 0..10 {
        let soul = format!("soul{}", i);
        let node = create_test_node(&soul, "value", &json!(i));
        storage.put(&soul, &node).await.unwrap();
    }

    for i in 0..10 {
        let soul = format!("soul{}", i);
        assert!(storage.has(&soul).await.unwrap());
        let node = storage.get(&soul).await.unwrap().unwrap();
        assert_eq!(node.data.get("value"), Some(&json!(i)));
    }
}

#[tokio::test]
async fn test_memory_storage_default() {
    let storage = MemoryStorage::default();
    let node = create_test_node("soul1", "name", &json!("Test"));
    storage.put("soul1", &node).await.unwrap();
    assert!(storage.has("soul1").await.unwrap());
}

// ========== SledStorage Tests ==========

#[tokio::test]
async fn test_sled_storage_new() {
    let temp_dir = std::env::temp_dir().join("gun_test_sled");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir); // Clean up if exists
    let storage = SledStorage::new(path).unwrap();

    assert!(!storage.has("test").await.unwrap());
    let _ = std::fs::remove_dir_all(&temp_dir); // Clean up
}

#[tokio::test]
async fn test_sled_storage_put_get() {
    let temp_dir = std::env::temp_dir().join("gun_test_sled_put_get");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = SledStorage::new(path).unwrap();

    let node = create_test_node("soul1", "name", &json!("Alice"));
    storage.put("soul1", &node).await.unwrap();

    let retrieved = storage.get("soul1").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_node = retrieved.unwrap();
    assert_eq!(retrieved_node.get_soul(), Some("soul1".to_string()));
    assert_eq!(retrieved_node.data.get("name"), Some(&json!("Alice")));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_sled_storage_persistence() {
    let temp_dir = std::env::temp_dir().join("gun_test_sled_persist");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Create storage and write data
    {
        let storage = SledStorage::new(path).unwrap();
        let node = create_test_node("soul1", "name", &json!("Alice"));
        storage.put("soul1", &node).await.unwrap();
    }

    // Create new storage instance (simulating restart)
    let storage2 = SledStorage::new(path).unwrap();
    let retrieved = storage2.get("soul1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().data.get("name"), Some(&json!("Alice")));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_sled_storage_has() {
    let temp_dir = std::env::temp_dir().join("gun_test_sled_has");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = SledStorage::new(path).unwrap();

    assert!(!storage.has("soul1").await.unwrap());
    let node = create_test_node("soul1", "name", &json!("Alice"));
    storage.put("soul1", &node).await.unwrap();
    assert!(storage.has("soul1").await.unwrap());
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_sled_storage_large_dataset() {
    let temp_dir = std::env::temp_dir().join("gun_test_sled_large");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = SledStorage::new(path).unwrap();

    // Store many nodes
    for i in 0..100 {
        let soul = format!("soul{}", i);
        let node = create_test_node(&soul, "value", &json!(i));
        storage.put(&soul, &node).await.unwrap();
    }

    // Verify all
    for i in 0..100 {
        let soul = format!("soul{}", i);
        assert!(storage.has(&soul).await.unwrap());
        let node = storage.get(&soul).await.unwrap().unwrap();
        assert_eq!(node.data.get("value"), Some(&json!(i)));
    }
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ========== LocalStorage Tests ==========

#[tokio::test]
async fn test_local_storage_new() {
    let temp_dir = std::env::temp_dir().join("gun_test_local");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = LocalStorage::new(path).unwrap();

    assert!(!storage.has("test").await.unwrap());
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_local_storage_put_get() {
    let temp_dir = std::env::temp_dir().join("gun_test_local_put_get");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = LocalStorage::new(path).unwrap();

    let node = create_test_node("soul1", "name", &json!("Alice"));
    storage.put("soul1", &node).await.unwrap();

    let retrieved = storage.get("soul1").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_node = retrieved.unwrap();
    assert_eq!(retrieved_node.get_soul(), Some("soul1".to_string()));
    assert_eq!(retrieved_node.data.get("name"), Some(&json!("Alice")));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_local_storage_persistence() {
    let temp_dir = std::env::temp_dir().join("gun_test_local_persist");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Create storage and write data
    {
        let storage = LocalStorage::new(path).unwrap();
        let node = create_test_node("soul1", "name", &json!("Alice"));
        storage.put("soul1", &node).await.unwrap();
    }

    // Create new storage instance (simulating restart)
    let storage2 = LocalStorage::new(path).unwrap();
    let retrieved = storage2.get("soul1").await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().data.get("name"), Some(&json!("Alice")));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_local_storage_has() {
    let temp_dir = std::env::temp_dir().join("gun_test_local_has");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = LocalStorage::new(path).unwrap();

    assert!(!storage.has("soul1").await.unwrap());
    let node = create_test_node("soul1", "name", &json!("Alice"));
    storage.put("soul1", &node).await.unwrap();
    assert!(storage.has("soul1").await.unwrap());
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_local_storage_update() {
    let temp_dir = std::env::temp_dir().join("gun_test_local_update");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = LocalStorage::new(path).unwrap();

    let node1 = create_test_node("soul1", "name", &json!("Alice"));
    let mut node2 = create_test_node("soul1", "name", &json!("Bob"));
    node2.data.insert("age".to_string(), json!(30));

    storage.put("soul1", &node1).await.unwrap();
    storage.put("soul1", &node2).await.unwrap();

    let retrieved = storage.get("soul1").await.unwrap().unwrap();
    assert_eq!(retrieved.data.get("name"), Some(&json!("Bob")));
    assert_eq!(retrieved.data.get("age"), Some(&json!(30)));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_local_storage_multiple_nodes() {
    let temp_dir = std::env::temp_dir().join("gun_test_local_multi");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = LocalStorage::new(path).unwrap();

    for i in 0..10 {
        let soul = format!("soul{}", i);
        let node = create_test_node(&soul, "value", &json!(i));
        storage.put(&soul, &node).await.unwrap();
    }

    for i in 0..10 {
        let soul = format!("soul{}", i);
        assert!(storage.has(&soul).await.unwrap());
        let node = storage.get(&soul).await.unwrap().unwrap();
        assert_eq!(node.data.get("value"), Some(&json!(i)));
    }
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_local_storage_special_characters_in_soul() {
    let temp_dir = std::env::temp_dir().join("gun_test_local_special");
    let path = temp_dir.to_str().unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);
    let storage = LocalStorage::new(path).unwrap();

    // Test with special characters in soul (should be URL-encoded)
    let soul = "soul/with/slashes";
    let node = create_test_node(soul, "name", &json!("Test"));
    storage.put(soul, &node).await.unwrap();

    let retrieved = storage.get(soul).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().data.get("name"), Some(&json!("Test")));
    let _ = std::fs::remove_dir_all(&temp_dir);
}
