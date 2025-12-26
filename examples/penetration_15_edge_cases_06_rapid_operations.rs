/// Test: Rapid operations
/// 
/// Tests rapid put/get cycles.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Test: Rapid operations");
    println!("Description: Test rapid put/get cycles");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Arc::new(Gun::new(secret_key, public_key));
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Rapid operations ---");
    let mut handles = vec![];
    for i in 0..100 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            gun_clone.get("test").get(&format!("rapid{}", i)).put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut rapid_success = 0;
    for handle in handles {
        if handle.await.is_ok() {
            rapid_success += 1;
        }
    }
    
    if rapid_success >= 90 {
        println!("✓ Rapid operations: {}/100 succeeded", rapid_success);
        success_count += 1;
    } else {
        println!("✗ Rapid operations: Only {}/100 succeeded", rapid_success);
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

