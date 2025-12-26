/// Test: Special characters
/// 
/// Tests special characters and unicode.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Special characters");
    println!("Description: Test special characters and unicode");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Special characters
    println!("\n--- Test: Special characters ---");
    let special = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
    match gun.get("test").get("special").put(json!(special)).await {
        Ok(_) => {
            println!("✓ Special characters: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Special characters: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test: Unicode
    println!("\n--- Test: Unicode ---");
    let unicode = "Hello 世界 🌍 こんにちは";
    match gun.get("test").get("unicode").put(json!(unicode)).await {
        Ok(_) => {
            println!("✓ Unicode: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Unicode: Failed - {}", e);
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

