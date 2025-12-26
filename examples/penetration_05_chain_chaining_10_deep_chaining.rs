/// Test: Chain method chaining - 10+ method chains
/// 
/// Tests very deep method chaining (10+ methods).

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - 10+ method chains");
    println!("Description: Test very deep method chaining");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: 10+ method chain
    println!("\n--- Test: 10+ method chain ---");
    let mut chain = gun.get("a");
    for i in 0..10 {
        chain = chain.get(&format!("level{}", i));
    }
    println!("✓ 10+ method chain: Success (no panic)");
    success_count += 1;
    
    // Test: 15 method chain with put
    println!("\n--- Test: 15 method chain with put ---");
    let mut chain = gun.get("deep");
    for i in 0..14 {
        chain = chain.get(&format!("l{}", i));
    }
    
    match chain.put(json!({"value": "deep"})).await {
        Ok(_) => {
            println!("✓ 15 method chain with put: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ 15 method chain with put: Failed - {}", e);
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

