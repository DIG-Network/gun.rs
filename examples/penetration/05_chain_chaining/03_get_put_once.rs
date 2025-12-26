/// Test: Chain method chaining - get().put().once() (two clients)
/// 
/// Tests chaining get(), put(), and once() methods.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().put().once() (two clients)");
    println!("Description: Test get().put().once() chaining");
    
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
    let test_key = format!("chained3_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Client1 get().put(), Client2 once()
    println!("\n--- Test: Client1 get().put(), Client2 once() ---");
    match client1.get("test").get(&test_key).put(json!({"value": 200})).await {
        Ok(_) => {
            println!("✓ Client1: get().put() - Success");
            tokio::time::sleep(Duration::from_millis(2000)).await;
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match client2.get("test").get(&test_key).once(move |data, _key| {
                if data.is_object() {
                    received_clone.store(true, Ordering::Relaxed);
                }
            }).await {
                Ok(_) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Client2: get().put().once() - Success");
                        success_count += 1;
                    } else {
                        println!("✗ Client2: get().put().once() - Data not received");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ Client2: get().put().once() - Failed - {}", e);
                    fail_count += 1;
                }
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
