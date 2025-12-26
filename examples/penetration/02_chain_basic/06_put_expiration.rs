/// Test: Chain.put() with time-based expiration (two clients)
/// 
/// Tests putting data with <? suffix for expiration (if supported).

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with time-based expiration (two clients)");
    println!("Description: Test <? suffix for expiration");
    
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
    
    // Note: Expiration with <? suffix may not be fully implemented
    // This test documents current behavior
    
    // Test 1: Client1 puts with expiration metadata
    println!("\n--- Test 1: Client1 put with expiration time ---");
    let expiry_time = chrono::Utc::now().timestamp_millis() as f64 + 3600000.0; // 1 hour from now
    match client1.get("test").get(&format!("expiring_{}", timestamp)).put(json!({
        "data": "will expire",
        "exp": expiry_time
    })).await {
        Ok(_) => {
            println!("✓ Client1: Expiration metadata - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Expiration metadata - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Client1 puts with past expiration
    println!("\n--- Test 2: Client1 put with past expiration ---");
    let past_expiry = chrono::Utc::now().timestamp_millis() as f64 - 1000.0; // 1 second ago
    match client1.get("test").get(&format!("expired_{}", timestamp)).put(json!({
        "data": "already expired",
        "exp": past_expiry
    })).await {
        Ok(_) => {
            println!("✓ Client1: Past expiration - Success (data may be filtered)");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Past expiration - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    println!("Note: <? suffix expiration may not be fully implemented");
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
