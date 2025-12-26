/// Test: Chain.on() - Concurrent subscriptions (two clients)
/// 
/// Tests concurrent subscription operations.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Concurrent subscriptions (two clients)");
    println!("Description: Test concurrent subscription operations");
    
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
    
    // Test: Client2 concurrent subscriptions to different keys, Client1 puts data
    println!("\n--- Test: Client2 concurrent subscriptions, Client1 puts ---");
    let mut handles = vec![];
    
    for i in 0..10 {
        let client2_clone = client2.clone();
        let client1_clone = client1.clone();
        let key = format!("concurrent{}_{}", i, timestamp);
        let handle = tokio::spawn(async move {
            let update_count = Arc::new(AtomicU32::new(0));
            let count_clone = update_count.clone();
            let key_clone = key.clone();
            
            let chain = client2_clone.get("test").get(&key_clone);
            chain.on(move |data, _key| {
                if data.is_i64() {
                    count_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            // Client1 puts data
            match client1_clone.get("test").get(&key).put(json!(i)).await {
                Ok(_) => {}
                Err(_) => return false,
            }
            
            tokio::time::sleep(Duration::from_millis(2000)).await;
            
            update_count.load(Ordering::Relaxed) > 0
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        match handle.await {
            Ok(true) => concurrent_success += 1,
            Ok(false) => fail_count += 1,
            Err(e) => {
                println!("✗ Join error: {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_success == 10 {
        println!("✓ Concurrent subscriptions: All 10 succeeded");
        success_count += 1;
    } else {
        println!("✗ Concurrent subscriptions: Only {}/10 succeeded", concurrent_success);
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
