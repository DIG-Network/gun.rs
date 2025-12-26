/// Test: Complete application workflow
/// 
/// Tests a complete application workflow.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Complete application workflow");
    println!("Description: Test a complete application workflow");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Complete workflow ---");
    
    // Step 1: Create data
    match gun.get("app").get("users").get("alice").put(json!({
        "name": "Alice",
        "age": 30
    })).await {
        Ok(_) => {
            println!("✓ Step 1: Data created");
            
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            // Step 2: Read data
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match gun.get("app").get("users").get("alice").once(move |data, _key| {
                if let Some(obj) = data.as_object() {
                    if obj.get("name").and_then(|v| v.as_str()) == Some("Alice") {
                        received_clone.store(true, Ordering::Relaxed);
                    }
                }
            }).await {
                Ok(_) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Step 2: Data read");
                        println!("✓ Complete workflow: Success");
                        success_count += 1;
                    } else {
                        println!("✗ Step 2: Data not received");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ Step 2: Read failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Step 1: Create failed - {}", e);
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

