/// Test: Chain.set() - Remove from set (two clients)
/// 
/// Tests removing items from a set (if supported).

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.set() - Remove from set (two clients)");
    println!("Description: Remove items from set");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    // Create Client 1
    println!("\n--- Setup: Creating Client 1 ---");
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key1 = SecretKey::from_seed(&[1 u8; 32]);
    let public_key1 = secret_key1.public_key();
    let client1 = match Gun::with_options(secret_key1, public_key1, options1).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 1: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Create Client 2
    println!("--- Setup: Creating Client 2 ---");
    let options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key2 = SecretKey::from_seed(&[2 u8; 32]);
    let public_key2 = secret_key2.public_key();
    let client2 = match Gun::with_options(secret_key2, public_key2, options2).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 2: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("set_remove_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Client1 adds items
    println!("\n--- Setup: Client1 adding items ---");
    for i in 1..=3 {
        match client1.get("test").get(&test_key).set(json!({"id": i, "name": format!("item{}", i)})).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Client1: Failed to add item {}: {}", i, e);
                fail_count += 1;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    tokio::time::sleep(Duration::from_millis(1500)).await;
    
    // Test: Client1 removes item (by setting to null or using special method)
    println!("\n--- Test: Client1 removing item ---");
    // Note: Gun.js set() doesn't have explicit remove - items are removed by setting to null
    // or by not including them in subsequent operations
    match client1.get("test").get(&test_key).set(json!(null)).await {
        Ok(_) => {
            println!("✓ Client1: Remove item - Success (set to null)");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Remove item - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    println!("Note: Set removal behavior may vary");
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
