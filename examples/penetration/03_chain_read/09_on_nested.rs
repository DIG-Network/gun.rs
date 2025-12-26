/// Test: Chain.on() - Subscribe to nested properties (two clients)
/// 
/// Tests subscribing to nested properties.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Subscribe to nested properties (two clients)");
    println!("Description: Subscribe to nested properties");
    
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
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Client1 puts nested data
    println!("\n--- Setup: Client1 putting nested data ---");
    let test_key1 = format!("nested_sub_{}", timestamp);
    match client1.get("test").get(&test_key1).put(json!({
        "level1": {
            "level2": 42
        }
    })).await {
        Ok(_) => println!("✓ Client1: Nested data put"),
        Err(e) => {
            println!("✗ Client1: Failed to put: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(3000)).await;
    
    // Test 1: Client2 subscribes to nested property
    println!("\n--- Test 1: Client2 subscribing to nested property ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    let key1_clone = test_key1.clone();
    
    client2.get("test").get(&key1_clone).get("level1").get("level2").on(move |data, _key| {
        if data.as_i64() == Some(42) {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(3000)).await;
    
    let count = update_count.load(Ordering::Relaxed);
    if count > 0 {
        println!("✓ Client2: Nested subscription - Received {} update(s)", count);
        success_count += 1;
    } else {
        println!("✗ Client2: Nested subscription - No updates received");
        fail_count += 1;
    }
    
    // Test 2: Client1 updates nested property, Client2 subscribes
    println!("\n--- Test 2: Client1 updates, Client2 subscribes ---");
    let test_key2 = format!("nested_sub2_{}", timestamp);
    let update_count2 = Arc::new(AtomicU32::new(0));
    let count_clone2 = update_count2.clone();
    let key2_clone = test_key2.clone();
    
    client2.get("test").get(&key2_clone).get("level1").get("level2").on(move |data, _key| {
        if data.is_i64() {
            count_clone2.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    match client1.get("test").get(&test_key2).get("level1").get("level2").put(json!(100)).await {
        Ok(_) => {
            println!("✓ Client1: Nested property updated");
            tokio::time::sleep(Duration::from_millis(3000)).await;
            
            let count2 = update_count2.load(Ordering::Relaxed);
            if count2 > 0 {
                println!("✓ Client2: Update received - {} update(s)", count2);
                success_count += 1;
            } else {
                println!("✗ Client2: Update not received");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Client1: Update failed: {}", e);
            fail_count += 1;
        }
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
