/// Test: Chain method chaining - get().put().on()
/// 
/// Tests chaining get(), put(), and on() methods.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().put().on()");
    println!("Description: Test get().put().on() chaining");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().put().on()
    println!("\n--- Test: get().put().on() ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    
    match gun.get("test").get("chained4").put(json!({"value": 300})).await {
        Ok(chain) => {
            chain.on(move |data, _key| {
                if data.is_object() {
                    count_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let count = update_count.load(Ordering::Relaxed);
            if count > 0 {
                println!("✓ get().put().on(): Success");
                success_count += 1;
            } else {
                println!("✗ get().put().on(): No updates received");
                fail_count += 1;
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

