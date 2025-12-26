/// Test: Complete application workflow (two clients)
/// 
/// Tests a complete application workflow.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    println!("Test: Complete application workflow (two clients)");
    println!("Description: Test a complete application workflow");
    
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
    let test_key = format!("alice_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Complete workflow ---");
    
    // Step 1: Client1 creates data
    match client1.get("app").get("users").get(&test_key).put(json!({
        "name": "Alice",
        "age": 30
    })).await {
        Ok(_) => {
            println!("✓ Client1: Step 1 - Data created");
            
            tokio::time::sleep(Duration::from_millis(2000)).await;
            
            // Step 2: Client2 reads data
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            let key_clone = test_key.clone();
            match timeout(Duration::from_secs(10), client2.get("app").get("users").get(&key_clone).once(move |data, _key| {
                if let Some(obj) = data.as_object() {
                    if obj.get("name").and_then(|v| v.as_str()) == Some("Alice") {
                        received_clone.store(true, Ordering::Relaxed);
                    }
                }
            })).await {
                Ok(Ok(_)) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Client2: Step 2 - Data read");
                        println!("✓ Complete workflow: Success");
                        success_count += 1;
                    } else {
                        println!("✗ Client2: Step 2 - Data not received");
                        fail_count += 1;
                    }
                }
                Ok(Err(e)) => {
                    println!("✗ Client2: Step 2 - Read failed - {}", e);
                    fail_count += 1;
                }
                Err(_) => {
                    println!("✗ Client2: Step 2 - Read timed out after 10 seconds");
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Client1: Step 1 - Create failed - {}", e);
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
