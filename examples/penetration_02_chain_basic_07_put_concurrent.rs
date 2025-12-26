/// Test: Chain.put() concurrent operations
/// 
/// Tests concurrent puts to same and different keys.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() concurrent operations");
    println!("Description: Test concurrent puts to same/different keys");
    
    let gun = Arc::new(Gun::new());
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Concurrent puts to different keys
    println!("\n--- Test 1: Concurrent puts to different keys ---");
    let mut handles = vec![];
    for i in 0..10 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            gun_clone.get(&format!("key{}", i)).put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => concurrent_success += 1,
            Ok(Err(e)) => {
                println!("✗ Concurrent different keys: Error - {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("✗ Concurrent different keys: Join error - {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_success == 10 {
        println!("✓ Concurrent different keys: All 10 succeeded");
        success_count += 1;
    }
    
    // Test 2: Concurrent puts to same key
    println!("\n--- Test 2: Concurrent puts to same key ---");
    let mut handles = vec![];
    for i in 0..10 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            gun_clone.get("same_key").put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut concurrent_same_success = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => concurrent_same_success += 1,
            Ok(Err(e)) => {
                println!("✗ Concurrent same key: Error - {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("✗ Concurrent same key: Join error - {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_same_success == 10 {
        println!("✓ Concurrent same key: All 10 succeeded");
        success_count += 1;
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

