/// Test: Circular references
/// 
/// Tests circular references.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Circular references");
    println!("Description: Test circular references");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Circular references ---");
    let node1_soul = "circular_node1";
    let node2_soul = "circular_node2";
    
    match gun.get(node1_soul).put(json!({
        "name": "Node 1",
        "friend": {"#": node2_soul}
    })).await {
        Ok(_) => {
            match gun.get(node2_soul).put(json!({
                "name": "Node 2",
                "friend": {"#": node1_soul}
            })).await {
                Ok(_) => {
                    println!("✓ Circular references: Success");
                    success_count += 1;
                }
                Err(e) => {
                    println!("✗ Circular references: Failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Circular references: Failed - {}", e);
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

