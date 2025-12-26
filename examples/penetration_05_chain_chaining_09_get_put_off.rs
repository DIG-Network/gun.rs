/// Test: Chain method chaining - get().on().put().off()
/// 
/// Tests chaining get(), on(), put(), and off() methods.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().on().put().off()");
    println!("Description: Test get().on().put().off() chaining");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().on().put().off()
    println!("\n--- Test: get().on().put().off() ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    
    let chain = gun.get("test").get("chained6");
    chain.on(move |data, _key| {
        if data.is_object() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    match chain.put(json!({"value": 500})).await {
        Ok(_) => {
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            let count_before = update_count.load(Ordering::Relaxed);
            chain.off();
            
            match chain.put(json!({"value": 600})).await {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    let count_after = update_count.load(Ordering::Relaxed);
                    
                    if count_before > 0 && count_after == count_before {
                        println!("✓ get().on().put().off(): Success");
                        success_count += 1;
                    } else {
                        println!("✗ get().on().put().off(): Off didn't work");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ get().on().put().off(): Second put failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ get().on().put().off(): Put failed - {}", e);
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

