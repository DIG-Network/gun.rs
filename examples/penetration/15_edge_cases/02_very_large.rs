/// Test: Very large data (two clients)
/// 
/// Tests very large data and very deep nesting.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Very large data (two clients)");
    println!("Description: Test very large data and very deep nesting");
    
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
    
    // Test: Client1 large string (100KB)
    println!("\n--- Test: Client1 large string (100KB) ---");
    let large_string = "x".repeat(100_000);
    match client1.get("test").get(&format!("large_{}", timestamp)).put(json!(large_string)).await {
        Ok(_) => {
            println!("✓ Client1: Large string - Success");
            tokio::time::sleep(Duration::from_millis(2000)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Large string - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test: Client1 deep nesting (10 levels)
    println!("\n--- Test: Client1 deep nesting (10 levels) ---");
    let mut chain = client1.get("test").get(&format!("deep_{}", timestamp));
    for i in 0..10 {
        chain = chain.get(&format!("level{}", i));
    }
    match chain.put(json!("deep")).await {
        Ok(_) => {
            println!("✓ Client1: Deep nesting - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Deep nesting - Failed - {}", e);
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
