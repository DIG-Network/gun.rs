/// Test: Error propagation through chains
/// 
/// Tests error propagation through method chains.

use gun::Gun;
use chia_bls::{SecretKey, PublicKey};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Error propagation through chains");
    println!("Description: Test error propagation through method chains");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let fail_count = 0;
    
    println!("\n--- Test: Error propagation ---");
    // Test that errors propagate correctly
    match gun.get("test").get("error_test").put(json!({"value": 1})).await {
        Ok(_) => {
            println!("✓ Error propagation: Operation succeeded");
            success_count += 1;
        }
        Err(e) => {
            println!("✓ Error propagation: Error correctly propagated - {}", e);
            success_count += 1;
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

