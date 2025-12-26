/// Test: Chain.once() - Concurrent reads
/// 
/// Tests concurrent read operations.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Concurrent reads");
    println!("Description: Test concurrent read operations");
    
    let gun = Arc::new(Gun::new());
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Put data
    println!("\n--- Setup: Put data ---");
    for i in 0..10 {
        match gun.get("test").get(&format!("key{}", i)).put(json!(i)).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Failed to put key{}: {}", i, e);
                fail_count += 1;
            }
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Test: Concurrent reads
    println!("\n--- Test: Concurrent reads ---");
    let mut handles = vec![];
    for i in 0..10 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match gun_clone.get("test").get(&format!("key{}", i)).once(move |data, _key| {
                if data.as_i64() == Some(i) {
                    received_clone.store(true, Ordering::Relaxed);
                }
            }).await {
                Ok(_) => received.load(Ordering::Relaxed),
                Err(_) => false,
            }
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        match handle.await {
            Ok(true) => concurrent_success += 1,
            Ok(false) => fail_count += 1,
            Err(e) => {
                println!("✗ Join error: {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_success == 10 {
        println!("✓ Concurrent reads: All 10 succeeded");
        success_count += 1;
    } else {
        println!("✗ Concurrent reads: Only {}/10 succeeded", concurrent_success);
        fail_count += 1;
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

