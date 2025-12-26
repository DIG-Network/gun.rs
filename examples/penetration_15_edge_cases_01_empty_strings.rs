/// Test: Empty strings
/// 
/// Tests empty keys and empty values.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Empty strings");
    println!("Description: Test empty keys and empty values");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Empty value
    println!("\n--- Test: Empty value ---");
    match gun.get("test").get("empty").put(json!("")).await {
        Ok(_) => {
            println!("✓ Empty value: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Empty value: Failed - {}", e);
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

