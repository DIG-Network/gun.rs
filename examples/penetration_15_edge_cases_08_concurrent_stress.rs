/// Test: High concurrency stress test
/// 
/// Tests high concurrency stress scenarios.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Test: High concurrency stress test");
    println!("Description: Test high concurrency stress scenarios");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Arc::new(Gun::new(secret_key, public_key));
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: High concurrency stress ---");
    let mut handles = vec![];
    for i in 0..1000 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            gun_clone.get("test").get(&format!("stress{}", i)).put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut stress_success = 0;
    for handle in handles {
        if handle.await.is_ok() {
            stress_success += 1;
        }
    }
    
    if stress_success >= 900 {
        println!("✓ High concurrency stress: {}/1000 succeeded", stress_success);
        success_count += 1;
    } else {
        println!("✗ High concurrency stress: Only {}/1000 succeeded", stress_success);
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

