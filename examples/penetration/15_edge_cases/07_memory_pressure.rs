/// Test: Operations under memory pressure (two clients)
/// 
/// Tests operations under memory pressure.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Operations under memory pressure (two clients)");
    println!("Description: Test operations under memory pressure");
    
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
    
    println!("\n--- Test: Client1 memory pressure ---");
    // Create many nodes to simulate memory pressure
    for i in 0..1000 {
        match client1.get("test").get(&format!("pressure{}_{}", i, timestamp)).put(json!(i)).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Client1: Memory pressure - Failed at {} - {}", i, e);
                fail_count += 1;
                break;
            }
        }
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    if fail_count == 0 {
        println!("✓ Client1: Memory pressure - Success (1000 operations)");
        success_count += 1;
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
