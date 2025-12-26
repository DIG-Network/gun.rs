/// Test: Chain.put() edge cases (two clients)
/// 
/// Tests empty strings, very long strings, special characters, unicode.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() edge cases (two clients)");
    println!("Description: Test empty strings, long strings, special chars, unicode");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    // Create Client 1
    println!("\n--- Setup: Creating Client 1 ---");
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let client1 = match Gun::with_options(options1).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 1: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Create Client 2
    println!("--- Setup: Creating Client 2 ---");
    let options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let client2 = match Gun::with_options(options2).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 2: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Empty string - Client1 puts
    println!("\n--- Test 1: Client1 empty string ---");
    match client1.get("test").get(&format!("empty_string_{}", timestamp)).put(json!("")).await {
        Ok(_) => {
            println!("✓ Client1: Empty string - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Empty string - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Very long string (1MB) - Client1 puts
    println!("\n--- Test 2: Client1 very long string (1MB) ---");
    let long_string = "x".repeat(1_000_000);
    match client1.get("test").get(&format!("long_string_{}", timestamp)).put(json!(long_string)).await {
        Ok(_) => {
            println!("✓ Client1: Very long string - Success");
            tokio::time::sleep(Duration::from_millis(2000)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Very long string - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Special characters - Client1 puts
    println!("\n--- Test 3: Client1 special characters ---");
    let special = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
    match client1.get("test").get(&format!("special_{}", timestamp)).put(json!(special)).await {
        Ok(_) => {
            println!("✓ Client1: Special characters - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Special characters - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 4: Unicode characters - Client1 puts
    println!("\n--- Test 4: Client1 unicode characters ---");
    let unicode = "Hello 世界 🌍 こんにちは مرحبا";
    match client1.get("test").get(&format!("unicode_{}", timestamp)).put(json!(unicode)).await {
        Ok(_) => {
            println!("✓ Client1: Unicode - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Unicode - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 5: Newlines and tabs - Client1 puts
    println!("\n--- Test 5: Client1 newlines and tabs ---");
    let with_newlines = "Line 1\nLine 2\tTabbed";
    match client1.get("test").get(&format!("newlines_{}", timestamp)).put(json!(with_newlines)).await {
        Ok(_) => {
            println!("✓ Client1: Newlines/tabs - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Newlines/tabs - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 6: Empty object - Client1 puts
    println!("\n--- Test 6: Client1 empty object ---");
    match client1.get("test").get(&format!("empty_obj_{}", timestamp)).put(json!({})).await {
        Ok(_) => {
            println!("✓ Client1: Empty object - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Empty object - Failed - {}", e);
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
