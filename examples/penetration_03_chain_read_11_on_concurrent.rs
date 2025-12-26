/// Test: Chain.on() - Concurrent subscriptions
/// 
/// Tests concurrent subscription operations.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Concurrent subscriptions");
    println!("Description: Test concurrent subscription operations");
    
    let gun = Arc::new(Gun::new());
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Concurrent subscriptions to different keys
    println!("\n--- Test: Concurrent subscriptions ---");
    let mut handles = vec![];
    
    for i in 0..10 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            let update_count = Arc::new(AtomicU32::new(0));
            let count_clone = update_count.clone();
            
            let chain = gun_clone.get("test").get(&format!("concurrent{}", i));
            chain.on(move |data, _key| {
                if data.is_i64() {
                    count_clone.fetch_add(1, Ordering::Relaxed);
                }
            });
            
            // Put data
            match chain.put(json!(i)).await {
                Ok(_) => {}
                Err(_) => return false,
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            update_count.load(Ordering::Relaxed) > 0
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
        println!("✓ Concurrent subscriptions: All 10 succeeded");
        success_count += 1;
    } else {
        println!("✗ Concurrent subscriptions: Only {}/10 succeeded", concurrent_success);
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

