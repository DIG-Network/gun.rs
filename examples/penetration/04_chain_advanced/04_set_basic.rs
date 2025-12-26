/// Test: Chain.set() - Add items to set (two clients)
/// 
/// Tests adding items to a set.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.set() - Add items to set (two clients)");
    println!("Description: Add items to set");
    
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
    let test_key = format!("set_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Client1 adds single item
    println!("\n--- Test 1: Client1 adding single item ---");
    match client1.get("test").get(&test_key).set(json!({"id": 1, "name": "item1"})).await {
        Ok(_) => {
            println!("✓ Client1: Add single item - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Add single item - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Client1 adds multiple items
    println!("\n--- Test 2: Client1 adding multiple items ---");
    let mut add_success = 0;
    for i in 2..=5 {
        match client1.get("test").get(&test_key).set(json!({"id": i, "name": format!("item{}", i)})).await {
            Ok(_) => add_success += 1,
            Err(e) => {
                println!("✗ Client1: Add item {} - Failed - {}", i, e);
                fail_count += 1;
            }
        }
    }
    
    if add_success == 4 {
        println!("✓ Client1: Add multiple items - Success");
        tokio::time::sleep(Duration::from_millis(1500)).await;
        success_count += 1;
    } else {
        println!("✗ Client1: Add multiple items - Only {}/4 succeeded", add_success);
        fail_count += 1;
    }
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
