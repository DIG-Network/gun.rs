/// Test: Chain method chaining - get().on().put()
/// 
/// Tests subscribing then writing (get().on().put()).

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().on().put()");
    println!("Description: Subscribe then write (get().on().put())");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().on().put()
    println!("\n--- Test: get().on().put() ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    
    let chain = gun.get("test").get("chained5");
    chain.on(move |data, _key| {
        if data.is_object() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    match chain.put(json!({"value": 400})).await {
        Ok(_) => {
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let count = update_count.load(Ordering::Relaxed);
            if count > 0 {
                println!("✓ get().on().put(): Success");
                success_count += 1;
            } else {
                println!("✗ get().on().put(): No updates received");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ get().on().put(): Put failed - {}", e);
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

