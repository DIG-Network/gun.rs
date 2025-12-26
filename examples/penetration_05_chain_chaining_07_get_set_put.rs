/// Test: Chain method chaining - get().set().put()
/// 
/// Tests chaining get(), set(), and put() methods.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().set().put()");
    println!("Description: Test get().set().put() chaining");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().set().put()
    println!("\n--- Test: get().set().put() ---");
    let chain = gun.get("test").get("set_chain");
    
    match chain.set(json!({"id": 1, "name": "item1"})).await {
        Ok(set_chain) => {
            match set_chain.put(json!({"extra": "data"})).await {
                Ok(_) => {
                    println!("✓ get().set().put(): Success");
                    success_count += 1;
                }
                Err(e) => {
                    println!("✗ get().set().put(): Put failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ get().set().put(): Set failed - {}", e);
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

