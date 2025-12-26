/// Test: Chain.set() - Add duplicate items
/// 
/// Tests adding duplicate items to a set.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.set() - Add duplicate items");
    println!("Description: Add duplicate items to set");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Add same item multiple times
    println!("\n--- Test: Add duplicate items ---");
    let item = json!({"id": 1, "name": "duplicate"});
    
    for i in 0..3 {
        match gun.get("test").get("set_dup").set(item.clone()).await {
            Ok(_) => {
                println!("✓ Add duplicate {}: Success", i + 1);
            }
            Err(e) => {
                println!("✗ Add duplicate {}: Failed - {}", i + 1, e);
                fail_count += 1;
            }
        }
    }
    
    if fail_count == 0 {
        println!("✓ Duplicate items: All succeeded");
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

