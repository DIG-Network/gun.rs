/// Test: Chain.on() - Multiple subscriptions
/// 
/// Tests multiple subscriptions to the same data.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Multiple subscriptions");
    println!("Description: Multiple subscriptions to same data");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Multiple subscriptions
    println!("\n--- Test: Multiple subscriptions ---");
    let count1 = Arc::new(AtomicU32::new(0));
    let count2 = Arc::new(AtomicU32::new(0));
    let count3 = Arc::new(AtomicU32::new(0));
    
    let c1 = count1.clone();
    gun.get("test").get("multi").on(move |data, _key| {
        if data.is_i64() {
            c1.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    let c2 = count2.clone();
    gun.get("test").get("multi").on(move |data, _key| {
        if data.is_i64() {
            c2.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    let c3 = count3.clone();
    gun.get("test").get("multi").on(move |data, _key| {
        if data.is_i64() {
            c3.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    // Put data multiple times
    for i in 0..5 {
        match gun.get("test").get("multi").put(json!(i)).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Put failed: {}", e);
                fail_count += 1;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let c1_val = count1.load(Ordering::Relaxed);
    let c2_val = count2.load(Ordering::Relaxed);
    let c3_val = count3.load(Ordering::Relaxed);
    
    println!("  Listener 1: {} updates", c1_val);
    println!("  Listener 2: {} updates", c2_val);
    println!("  Listener 3: {} updates", c3_val);
    
    if c1_val > 0 && c2_val > 0 && c3_val > 0 {
        println!("✓ Multiple subscriptions: All listeners received updates");
        success_count += 1;
    } else {
        println!("✗ Multiple subscriptions: Some listeners didn't receive updates");
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

