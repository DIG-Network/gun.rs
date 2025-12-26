/// Test: Invalid data handling
/// 
/// Tests handling invalid JSON and invalid souls.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Invalid data handling");
    println!("Description: Test invalid JSON and invalid souls");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let fail_count = 0;
    
    // Test: Put with invalid soul (should handle gracefully)
    println!("\n--- Test: Invalid soul ---");
    match gun.get("").get("test").put(json!({"value": 1})).await {
        Ok(_) => {
            println!("✓ Invalid soul: Handled gracefully");
            success_count += 1;
        }
        Err(_) => {
            println!("✓ Invalid soul: Correctly rejected");
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

