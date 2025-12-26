/// Test: Chain.set() - Add duplicate items (two clients)
/// 
/// Tests adding duplicate items to a set.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.set() - Add duplicate items (two clients)");
    println!("Description: Add duplicate items to set");
    
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
    let test_key = format!("set_dup_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Client1 adds same item multiple times
    println!("\n--- Test: Client1 adding duplicate items ---");
    let item = json!({"id": 1, "name": "duplicate"});
    
    for i in 0..3 {
        match client1.get("test").get(&test_key).set(item.clone()).await {
            Ok(_) => {
                println!("✓ Client1: Add duplicate {} - Success", i + 1);
            }
            Err(e) => {
                println!("✗ Client1: Add duplicate {} - Failed - {}", i + 1, e);
                fail_count += 1;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    if fail_count == 0 {
        println!("✓ Client1: Duplicate items - All succeeded");
        tokio::time::sleep(Duration::from_millis(1500)).await;
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
