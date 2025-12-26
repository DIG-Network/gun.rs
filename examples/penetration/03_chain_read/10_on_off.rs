/// Test: Chain.on() then Chain.off() - Subscribe then unsubscribe (two clients)
/// 
/// Tests subscribing and then unsubscribing.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() then Chain.off() - Subscribe then unsubscribe (two clients)");
    println!("Description: Subscribe then unsubscribe");
    
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
    let test_key = format!("onoff_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Client2 subscribes, Client1 puts data, then Client2 unsubscribes
    println!("\n--- Test: Client2 subscribes, Client1 puts, Client2 unsubscribes ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    let key_clone = test_key.clone();
    
    let chain = client2.get("test").get(&key_clone);
    chain.on(move |data, _key| {
        if data.is_i64() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Client1 puts data - should trigger subscription
    match client1.get("test").get(&test_key).put(json!(1)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Client1: Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    let count_before = update_count.load(Ordering::Relaxed);
    println!("  Client2: Updates before unsubscribe: {}", count_before);
    
    // Client2 unsubscribes
    chain.off();
    
    // Client1 puts more data - should NOT trigger subscription
    match client1.get("test").get(&test_key).put(json!(2)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Client1: Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    let count_after = update_count.load(Ordering::Relaxed);
    println!("  Client2: Updates after unsubscribe: {}", count_after);
    
    if count_before > 0 && count_after == count_before {
        println!("✓ Subscribe/unsubscribe: Working correctly");
        success_count += 1;
    } else {
        println!("✗ Subscribe/unsubscribe: Not working correctly");
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
