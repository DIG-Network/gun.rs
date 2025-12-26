/// Test: Chain.on() - Subscribe to nested properties
/// 
/// Tests subscribing to nested properties.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Subscribe to nested properties");
    println!("Description: Subscribe to nested properties");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Put nested data
    println!("\n--- Setup: Put nested data ---");
    match gun.get("test").get("nested_sub").put(json!({
        "level1": {
            "level2": 42
        }
    })).await {
        Ok(_) => println!("✓ Nested data put"),
        Err(e) => {
            println!("✗ Failed to put: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Test 1: Subscribe to nested property
    println!("\n--- Test 1: Subscribe to nested property ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    
    gun.get("test").get("nested_sub").get("level1").get("level2").on(move |data, _key| {
        if data.as_i64() == Some(42) {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let count = update_count.load(Ordering::Relaxed);
    if count > 0 {
        println!("✓ Nested subscription: Received {} update(s)", count);
        success_count += 1;
    } else {
        println!("✗ Nested subscription: No updates received");
        fail_count += 1;
    }
    
    // Test 2: Update nested property
    println!("\n--- Test 2: Update nested property ---");
    let update_count2 = Arc::new(AtomicU32::new(0));
    let count_clone2 = update_count2.clone();
    
    gun.get("test").get("nested_sub2").get("level1").get("level2").on(move |data, _key| {
        if data.is_i64() {
            count_clone2.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    match gun.get("test").get("nested_sub2").get("level1").get("level2").put(json!(100)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let count2 = update_count2.load(Ordering::Relaxed);
    if count2 > 0 {
        println!("✓ Update nested: Received {} update(s)", count2);
        success_count += 1;
    } else {
        println!("✗ Update nested: No updates received");
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

