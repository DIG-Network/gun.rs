/// Test: Chain method chaining - get().put()
/// 
/// Tests chaining get() and put() methods.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().put()");
    println!("Description: Test get().put() chaining");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().put()
    println!("\n--- Test: get().put() ---");
    match gun.get("test").get("chained").put(json!({"value": 42})).await {
        Ok(_) => {
            println!("✓ get().put(): Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ get().put(): Failed - {}", e);
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

