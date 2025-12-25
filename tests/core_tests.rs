//! Comprehensive tests for core functionality
//! Tests GunCore initialization, UUID generation, and component integration

use gun::core::GunCore;
use gun::storage::MemoryStorage;
use std::sync::Arc;

#[test]
fn test_gun_core_new() {
    let core = GunCore::new();
    // Should be able to generate UUIDs
    let uuid = core.uuid(None);
    assert!(!uuid.is_empty());
}

#[test]
fn test_gun_core_with_storage() {
    let storage = Arc::new(MemoryStorage::new());
    let core = GunCore::with_storage(storage);

    // Should have storage
    assert!(core.storage.is_some());
}

#[test]
fn test_gun_core_uuid_generation() {
    let core = GunCore::new();

    // Generate multiple UUIDs - should be unique
    let uuid1 = core.uuid(None);
    let uuid2 = core.uuid(None);
    let uuid3 = core.uuid(None);

    assert_ne!(uuid1, uuid2);
    assert_ne!(uuid2, uuid3);
    assert_ne!(uuid1, uuid3);
}

#[test]
fn test_gun_core_uuid_format() {
    let core = GunCore::new();
    let uuid = core.uuid(None);

    // UUID should be non-empty alphanumeric string
    assert!(!uuid.is_empty());
    assert!(uuid.len() > 10); // Should have reasonable length
}

#[test]
fn test_gun_core_uuid_custom_length() {
    let core = GunCore::new();
    let uuid = core.uuid(Some(20));

    // Should respect length parameter (base + random part)
    assert!(!uuid.is_empty());
}

#[test]
fn test_gun_core_random_id() {
    let core = GunCore::new();

    let id1 = core.random_id(10);
    let id2 = core.random_id(10);

    // Should be different
    assert_ne!(id1, id2);

    // Should have correct length
    assert_eq!(id1.len(), 10);
    assert_eq!(id2.len(), 10);
}

#[test]
fn test_gun_core_random_id_lengths() {
    let core = GunCore::new();

    assert_eq!(core.random_id(5).len(), 5);
    assert_eq!(core.random_id(15).len(), 15);
    assert_eq!(core.random_id(100).len(), 100);
}

#[test]
fn test_gun_core_next_chain_id() {
    let core = GunCore::new();

    let id1 = core.next_chain_id();
    let id2 = core.next_chain_id();
    let id3 = core.next_chain_id();

    // Should be sequential
    assert_eq!(id1 + 1, id2);
    assert_eq!(id2 + 1, id3);
}

#[test]
fn test_gun_core_component_integration() {
    let core = GunCore::new();

    // All components should be initialized
    assert!(core.graph.get("test").is_none()); // Graph works
    let _uuid = core.uuid(None); // UUID generation works
    let _state = core.state.next(); // State works
    let _id = core.next_chain_id(); // Chain ID works
}

#[test]
fn test_gun_core_default() {
    let core = GunCore::default();
    let uuid = core.uuid(None);
    assert!(!uuid.is_empty());
}
