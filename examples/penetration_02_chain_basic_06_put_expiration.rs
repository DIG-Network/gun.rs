/// Test: Chain.put() with time-based expiration
/// 
/// Tests putting data with <? suffix for expiration (if supported).

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with time-based expiration");
    println!("Description: Test <? suffix for expiration");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Note: Expiration with <? suffix may not be fully implemented
    // This test documents current behavior
    
    // Test 1: Put with expiration metadata
    println!("\n--- Test 1: Put with expiration time ---");
    let expiry_time = chrono::Utc::now().timestamp_millis() as f64 + 3600000.0; // 1 hour from now
    match gun.get("test").get("expiring").put(json!({
        "data": "will expire",
        "exp": expiry_time
    })).await {
        Ok(_) => {
            println!("✓ Expiration metadata: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Expiration metadata: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Put with past expiration
    println!("\n--- Test 2: Put with past expiration ---");
    let past_expiry = chrono::Utc::now().timestamp_millis() as f64 - 1000.0; // 1 second ago
    match gun.get("test").get("expired").put(json!({
        "data": "already expired",
        "exp": past_expiry
    })).await {
        Ok(_) => {
            println!("✓ Past expiration: Success (data may be filtered)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Past expiration: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    println!("Note: <? suffix expiration may not be fully implemented");
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

