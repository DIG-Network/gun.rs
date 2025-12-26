/// Test: Chain.set() - Add items to set
/// 
/// Tests adding items to a set.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.set() - Add items to set");
    println!("Description: Add items to set");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Add single item
    println!("\n--- Test 1: Add single item ---");
    match gun.get("test").get("set").set(json!({"id": 1, "name": "item1"})).await {
        Ok(_) => {
            println!("✓ Add single item: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Add single item: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Add multiple items
    println!("\n--- Test 2: Add multiple items ---");
    for i in 2..=5 {
        match gun.get("test").get("set").set(json!({"id": i, "name": format!("item{}", i)})).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Add item {}: Failed - {}", i, e);
                fail_count += 1;
            }
        }
    }
    
    if fail_count == 0 {
        println!("✓ Add multiple items: Success");
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

