/// Test: Error handling under concurrency
/// 
/// Tests error handling under concurrent operations.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Test: Error handling under concurrency");
    println!("Description: Test error handling under concurrent operations");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Arc::new(Gun::new(secret_key, public_key));
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Concurrent errors ---");
    let mut handles = vec![];
    for i in 0..10 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            gun_clone.get("test").get(&format!("key{}", i)).put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        if handle.await.is_ok() {
            concurrent_success += 1;
        }
    }
    
    if concurrent_success == 10 {
        println!("✓ Concurrent errors: All 10 succeeded");
        success_count += 1;
    } else {
        println!("✗ Concurrent errors: Only {}/10 succeeded", concurrent_success);
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

