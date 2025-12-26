/// Test: Chain method chaining - get().put().get()
/// 
/// Tests chaining get(), put(), and get() methods.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().put().get()");
    println!("Description: Test get().put().get() chaining");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().put().get()
    println!("\n--- Test: get().put().get() ---");
    match gun.get("test").get("chained2").put(json!({"value": 100})).await {
        Ok(chain) => {
            // Chain continues with get()
            let _next_chain = chain.get("value");
            println!("✓ get().put().get(): Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ get().put().get(): Failed - {}", e);
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

