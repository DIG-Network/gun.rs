/// Test: Chain method chaining - get().put().on() (two clients)
/// 
/// Tests chaining get(), put(), and on() methods.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().put().on() (two clients)");
    println!("Description: Test get().put().on() chaining");
    
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
    let test_key = format!("chained4_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Client1 get().put(), Client2 on()
    println!("\n--- Test: Client1 get().put(), Client2 on() ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    let key_clone = test_key.clone();
    
    match client1.get("test").get(&test_key).put(json!({"value": 300})).await {
        Ok(_) => {
            println!("✓ Client1: get().put() - Success");
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            client2.get("test").get(&key_clone).on(move |data, _key| {
                if data.is_object() {
                    count_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            
            tokio::time::sleep(Duration::from_millis(2000)).await;
            
            let count = update_count.load(Ordering::Relaxed);
            if count > 0 {
                println!("✓ Client2: get().put().on() - Success");
                success_count += 1;
            } else {
                println!("✗ Client2: get().put().on() - No updates received");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Client1: get().put() - Failed - {}", e);
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
