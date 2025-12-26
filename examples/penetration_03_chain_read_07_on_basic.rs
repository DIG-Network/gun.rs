/// Test: Chain.on() - Subscribe to updates
/// 
/// Tests basic subscription to data updates.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Subscribe to updates");
    println!("Description: Basic subscription to data updates");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Subscribe then put
    println!("\n--- Test 1: Subscribe then put ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    
    gun.get("test").get("subscribe").on(move |data, _key| {
        if data.is_i64() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    // Put data
    match gun.get("test").get("subscribe").put(json!(1)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let count = update_count.load(Ordering::Relaxed);
    if count > 0 {
        println!("✓ Subscribe then put: Received {} update(s)", count);
        success_count += 1;
    } else {
        println!("✗ Subscribe then put: No updates received");
        fail_count += 1;
    }
    
    // Test 2: Put then subscribe (should get current value)
    println!("\n--- Test 2: Put then subscribe ---");
    let update_count2 = Arc::new(AtomicU32::new(0));
    let count_clone2 = update_count2.clone();
    
    match gun.get("test").get("subscribe2").put(json!(2)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    gun.get("test").get("subscribe2").on(move |data, _key| {
        if data.is_i64() {
            count_clone2.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let count2 = update_count2.load(Ordering::Relaxed);
    if count2 > 0 {
        println!("✓ Put then subscribe: Received {} update(s)", count2);
        success_count += 1;
    } else {
        println!("✗ Put then subscribe: No updates received");
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

