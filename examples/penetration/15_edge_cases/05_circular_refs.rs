/// Test: Circular references (two clients)
/// 
/// Tests circular references.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Circular references (two clients)");
    println!("Description: Test circular references");
    
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
    let node1_soul = format!("circular_node1_{}", timestamp);
    let node2_soul = format!("circular_node2_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Client1 circular references ---");
    match client1.get(&node1_soul).put(json!({
        "name": "Node 1",
        "friend": {"#": node2_soul}
    })).await {
        Ok(_) => {
            match client1.get(&node2_soul).put(json!({
                "name": "Node 2",
                "friend": {"#": node1_soul}
            })).await {
                Ok(_) => {
                    println!("✓ Client1: Circular references - Success");
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    success_count += 1;
                }
                Err(e) => {
                    println!("✗ Client1: Circular references - Failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Client1: Circular references - Failed - {}", e);
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
