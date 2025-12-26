/// Test: Very large data
/// 
/// Tests very large data and very deep nesting.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Very large data");
    println!("Description: Test very large data and very deep nesting");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Large string (100KB)
    println!("\n--- Test: Large string (100KB) ---");
    let large_string = "x".repeat(100_000);
    match gun.get("test").get("large").put(json!(large_string)).await {
        Ok(_) => {
            println!("✓ Large string: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Large string: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test: Deep nesting (10 levels)
    println!("\n--- Test: Deep nesting (10 levels) ---");
    let mut chain = gun.get("test");
    for i in 0..10 {
        chain = chain.get(&format!("level{}", i));
    }
    match chain.put(json!("deep")).await {
        Ok(_) => {
            println!("✓ Deep nesting: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Deep nesting: Failed - {}", e);
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

