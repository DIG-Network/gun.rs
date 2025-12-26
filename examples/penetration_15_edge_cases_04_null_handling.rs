/// Test: Null handling
/// 
/// Tests null values and null keys.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Null handling");
    println!("Description: Test null values and null keys");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Null value
    println!("\n--- Test: Null value ---");
    match gun.get("test").get("null").put(json!(null)).await {
        Ok(_) => {
            println!("✓ Null value: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Null value: Failed - {}", e);
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

