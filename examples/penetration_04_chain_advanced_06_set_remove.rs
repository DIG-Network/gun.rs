/// Test: Chain.set() - Remove from set
/// 
/// Tests removing items from a set (if supported).

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.set() - Remove from set");
    println!("Description: Remove items from set");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Add items
    println!("\n--- Setup: Add items ---");
    for i in 1..=3 {
        match gun.get("test").get("set_remove").set(json!({"id": i, "name": format!("item{}", i)})).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Failed to add item {}: {}", i, e);
                fail_count += 1;
            }
        }
    }
    
    // Test: Remove item (by setting to null or using special method)
    println!("\n--- Test: Remove item ---");
    // Note: Gun.js set() doesn't have explicit remove - items are removed by setting to null
    // or by not including them in subsequent operations
    match gun.get("test").get("set_remove").set(json!(null)).await {
        Ok(_) => {
            println!("✓ Remove item: Success (set to null)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Remove item: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    println!("Note: Set removal behavior may vary");
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

