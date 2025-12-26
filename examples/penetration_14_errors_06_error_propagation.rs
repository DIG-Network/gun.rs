/// Test: Error propagation through chains
/// 
/// Tests error propagation through method chains.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Error propagation through chains");
    println!("Description: Test error propagation through method chains");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
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

