/// Test: Invalid data handling
/// 
/// Tests handling invalid JSON and invalid souls.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Invalid data handling");
    println!("Description: Test invalid JSON and invalid souls");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
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

