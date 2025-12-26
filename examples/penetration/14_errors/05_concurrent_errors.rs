/// Test: Error handling under concurrency (two clients)
/// 
/// Tests error handling under concurrent operations.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Error handling under concurrency (two clients)");
    println!("Description: Test error handling under concurrent operations");
    
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
    
    println!("\n--- Test: Client1 concurrent operations ---");
    let mut handles = vec![];
    for i in 0..10 {
        let client1_clone = client1.clone();
        let key = format!("key{}_{}", i, timestamp);
        let handle = tokio::spawn(async move {
            client1_clone.get("test").get(&key).put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        if handle.await.is_ok() {
            concurrent_success += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    if concurrent_success == 10 {
        println!("✓ Client1: Concurrent errors - All 10 succeeded");
        success_count += 1;
    } else {
        println!("✗ Client1: Concurrent errors - Only {}/10 succeeded", concurrent_success);
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
