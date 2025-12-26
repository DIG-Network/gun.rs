//! Comprehensive tests for Gun instance
//! Tests initialization, options, and integration

use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

#[tokio::test]
async fn test_gun_new() {
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    // Should be able to get a chain
    let chain = gun.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_default() {
    // Note: Default implementation removed, using new() instead
    let secret_key = SecretKey::from_seed(&[1u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let chain = gun.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_with_options_default() {
    let secret_key = SecretKey::from_seed(&[2u8; 32]);
    let public_key = secret_key.public_key();
    let options = GunOptions::default();
    let gun = Gun::with_options(secret_key, public_key, options).await.unwrap();

    let chain = gun.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_with_options_storage() {
    use std::env;
    let temp_dir = env::temp_dir().join("gun_test_instance");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let secret_key = SecretKey::from_seed(&[3u8; 32]);
    let public_key = secret_key.public_key();
    let mut options = GunOptions {
        localStorage: true,
        ..Default::default()
    };
    options.storage_path = Some(temp_dir.to_str().unwrap().to_string());

    let gun = Gun::with_options(secret_key, public_key, options).await.unwrap();

    let chain = gun.get("test");
    chain.put(serde_json::json!("value")).await.unwrap();

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_gun_root() {
    let secret_key = SecretKey::from_seed(&[4u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let root = gun.root();

    // Root should allow getting chains
    let chain = root.get("test");
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}

#[tokio::test]
async fn test_gun_get() {
    let secret_key = SecretKey::from_seed(&[5u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let chain = gun.get("key");

    // Should be able to put data
    assert!(chain.put(serde_json::json!("value")).await.is_ok());
}
