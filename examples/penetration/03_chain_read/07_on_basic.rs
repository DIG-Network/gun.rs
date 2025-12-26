/// Test: Chain.on() - Subscribe to updates (two clients)
/// 
/// Tests basic subscription to data updates.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Subscribe to updates (two clients)");
    println!("Description: Basic subscription to data updates");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    // Create Client 1
    println!("\n--- Setup: Creating Client 1 ---");
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let client1 = match Gun::with_options(options1).await {
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
    let client2 = match Gun::with_options(options2).await {
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
    
    // Test 1: Client2 subscribes, then Client1 puts
    println!("\n--- Test 1: Client2 subscribes, Client1 puts ---");
    let test_key1 = format!("subscribe_{}", timestamp);
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    let key1_clone = test_key1.clone();
    
    client2.get("test").get(&key1_clone).on(move |data, _key| {
        if data.is_i64() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Client1 puts data
    match client1.get("test").get(&test_key1).put(json!(1)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Client1: Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    let count = update_count.load(Ordering::Relaxed);
    if count > 0 {
        println!("✓ Client2: Subscribe then put - Received {} update(s)", count);
        success_count += 1;
    } else {
        println!("✗ Client2: Subscribe then put - No updates received");
        fail_count += 1;
    }
    
    // Test 2: Client1 puts, then Client2 subscribes (should get current value)
    println!("\n--- Test 2: Client1 puts, Client2 subscribes ---");
    let test_key2 = format!("subscribe2_{}", timestamp);
    let update_count2 = Arc::new(AtomicU32::new(0));
    let count_clone2 = update_count2.clone();
    let key2_clone = test_key2.clone();
    
    match client1.get("test").get(&test_key2).put(json!(2)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Client1: Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    client2.get("test").get(&key2_clone).on(move |data, _key| {
        if data.is_i64() {
            count_clone2.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    let count2 = update_count2.load(Ordering::Relaxed);
    if count2 > 0 {
        println!("✓ Client2: Put then subscribe - Received {} update(s)", count2);
        success_count += 1;
    } else {
        println!("✗ Client2: Put then subscribe - No updates received");
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
