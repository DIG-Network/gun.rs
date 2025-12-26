/// Test: Chain method chaining - get().map().put() (two clients)
/// 
/// Tests chaining get(), map(), and put() methods.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().map().put() (two clients)");
    println!("Description: Test get().map().put() chaining");
    
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
    let test_key = format!("map_chain_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Client1 puts initial data
    match client1.get("test").get(&test_key).put(json!({"a": 1, "b": 2})).await {
        Ok(_) => {
            println!("✓ Client1: Initial data put");
            tokio::time::sleep(Duration::from_millis(2000)).await;
        }
        Err(e) => {
            println!("✗ Client1: Setup failed: {}", e);
            fail_count += 1;
        }
    }
    
    // Test: Client2 get().map().put()
    println!("\n--- Test: Client2 get().map().put() ---");
    let properties = Arc::new(Mutex::new(Vec::new()));
    let props_clone = properties.clone();
    let key_clone = test_key.clone();
    
    let chain = client2.get("test").get(&key_clone);
    chain.map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    match client1.get("test").get(&test_key).put(json!({"c": 3})).await {
        Ok(_) => {
            println!("✓ Client1: Additional data put");
            tokio::time::sleep(Duration::from_millis(2000)).await;
            let props = properties.lock().unwrap();
            if props.len() >= 2 {
                println!("✓ Client2: get().map().put() - Success");
                success_count += 1;
            } else {
                println!("✗ Client2: get().map().put() - Map didn't work");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Client1: get().map().put() - Put failed - {}", e);
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
