//! Comprehensive tests for Gun instance
//! Tests initialization, options, and integration

use gun::{Gun, GunOptions};

#[tokio::test]
async fn test_gun_new() {
    let gun = Gun::new();
    // Should be able to get a chain
    let chain = gun.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_default() {
    let gun = Gun::default();
    let chain = gun.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_with_options_default() {
    let options = GunOptions::default();
    let gun = Gun::with_options(options).await.unwrap();

    let chain = gun.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_with_options_storage() {
    use std::env;
    let temp_dir = env::temp_dir().join("gun_test_instance");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut options = GunOptions {
        localStorage: true,
        ..Default::default()
    };
    options.storage_path = Some(temp_dir.to_str().unwrap().to_string());

    let gun = Gun::with_options(options).await.unwrap();

    let chain = gun.get("test");
    chain.put(serde_json::json!("value")).await.unwrap();

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_gun_root() {
    let gun = Gun::new();
    let root = gun.root();

    // Root should allow getting chains
    let chain = root.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_get() {
    let gun = Gun::new();
    let chain = gun.get("key");

    // Should be able to put data
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}
