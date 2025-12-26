/// Test: Operations under memory pressure
/// 
/// Tests operations under memory pressure.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Operations under memory pressure");
    println!("Description: Test operations under memory pressure");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Memory pressure ---");
    // Create many nodes to simulate memory pressure
    for i in 0..1000 {
        match gun.get("test").get(&format!("pressure{}", i)).put(json!(i)).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Memory pressure: Failed at {} - {}", i, e);
                fail_count += 1;
                break;
            }
        }
    }
    
    if fail_count == 0 {
        println!("✓ Memory pressure: Success (1000 operations)");
        success_count += 1;
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

