/// Test: Chain method chaining - get().put().once()
/// 
/// Tests chaining get(), put(), and once() methods.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().put().once()");
    println!("Description: Test get().put().once() chaining");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().put().once()
    println!("\n--- Test: get().put().once() ---");
    match gun.get("test").get("chained3").put(json!({"value": 200})).await {
        Ok(chain) => {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match chain.once(move |data, _key| {
                if data.is_object() {
                    received_clone.store(true, Ordering::Relaxed);
                }
            }).await {
                Ok(_) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ get().put().once(): Success");
                        success_count += 1;
                    } else {
                        println!("✗ get().put().once(): Data not received");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ get().put().once(): Failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ get().put(): Failed - {}", e);
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

