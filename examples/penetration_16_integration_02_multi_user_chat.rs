/// Test: Multi-user chat simulation
/// 
/// Tests a multi-user chat simulation.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Multi-user chat simulation");
    println!("Description: Test a multi-user chat simulation");
    
    let gun = Arc::new(Gun::new());
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Multi-user chat ---");
    
    // Simulate multiple users sending messages
    let mut handles = vec![];
    for i in 0..5 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            gun_clone.get("chat").get("messages").get(&format!("msg{}", i)).put(json!({
                "user": format!("user{}", i),
                "message": format!("Hello from user{}", i),
                "timestamp": i
            })).await
        });
        handles.push(handle);
    }
    
    let mut chat_success = 0;
    for handle in handles {
        if handle.await.is_ok() {
            chat_success += 1;
        }
    }
    
    if chat_success == 5 {
        println!("✓ Multi-user chat: All 5 messages sent");
        success_count += 1;
    } else {
        println!("✗ Multi-user chat: Only {}/5 messages sent", chat_success);
        fail_count += 1;
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

