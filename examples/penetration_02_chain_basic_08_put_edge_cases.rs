/// Test: Chain.put() edge cases
/// 
/// Tests empty strings, very long strings, special characters, unicode.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() edge cases");
    println!("Description: Test empty strings, long strings, special chars, unicode");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Empty string
    println!("\n--- Test 1: Empty string ---");
    match gun.get("test").get("empty_string").put(json!("")).await {
        Ok(_) => {
            println!("✓ Empty string: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Empty string: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Very long string (1MB)
    println!("\n--- Test 2: Very long string (1MB) ---");
    let long_string = "x".repeat(1_000_000);
    match gun.get("test").get("long_string").put(json!(long_string)).await {
        Ok(_) => {
            println!("✓ Very long string: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Very long string: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Special characters
    println!("\n--- Test 3: Special characters ---");
    let special = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
    match gun.get("test").get("special").put(json!(special)).await {
        Ok(_) => {
            println!("✓ Special characters: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Special characters: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 4: Unicode characters
    println!("\n--- Test 4: Unicode characters ---");
    let unicode = "Hello 世界 🌍 こんにちは مرحبا";
    match gun.get("test").get("unicode").put(json!(unicode)).await {
        Ok(_) => {
            println!("✓ Unicode: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Unicode: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 5: Newlines and tabs
    println!("\n--- Test 5: Newlines and tabs ---");
    let with_newlines = "Line 1\nLine 2\tTabbed";
    match gun.get("test").get("newlines").put(json!(with_newlines)).await {
        Ok(_) => {
            println!("✓ Newlines/tabs: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Newlines/tabs: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 6: Empty object
    println!("\n--- Test 6: Empty object ---");
    match gun.get("test").get("empty_obj").put(json!({})).await {
        Ok(_) => {
            println!("✓ Empty object: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Empty object: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 7: Empty array
    println!("\n--- Test 7: Empty array ---");
    match gun.get("test").get("empty_arr").put(json!([])).await {
        Ok(_) => {
            println!("✓ Empty array: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Empty array: Failed - {}", e);
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

