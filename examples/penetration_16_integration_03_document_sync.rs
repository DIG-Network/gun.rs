/// Test: Document synchronization
/// 
/// Tests document synchronization.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Document synchronization");
    println!("Description: Test document synchronization");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Document synchronization ---");
    match gun.get("documents").get("doc1").put(json!({
        "title": "Test Document",
        "content": "This is test content",
        "version": 1
    })).await {
        Ok(_) => {
            println!("✓ Document synchronization: Document created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Document synchronization: Failed - {}", e);
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

