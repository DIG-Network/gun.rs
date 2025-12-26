/// Test: Chain.put() with nested structures
/// 
/// Tests putting nested objects and arrays of objects.

use gun::Gun;
use chia_bls::{SecretKey, PublicKey};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with nested structures");
    println!("Description: Test nested objects and arrays of objects");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Nested object (2 levels)
    println!("\n--- Test 1: Nested object (2 levels) ---");
    match gun.get("test").get("nested2").put(json!({
        "level1": {
            "level2": "value"
        }
    })).await {
        Ok(_) => {
            println!("✓ Nested 2 levels: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Nested 2 levels: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Nested object (3 levels)
    println!("\n--- Test 2: Nested object (3 levels) ---");
    match gun.get("test").get("nested3").put(json!({
        "level1": {
            "level2": {
                "level3": "value"
            }
        }
    })).await {
        Ok(_) => {
            println!("✓ Nested 3 levels: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Nested 3 levels: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Array of objects
    println!("\n--- Test 3: Array of objects ---");
    match gun.get("test").get("array_objects").put(json!([
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ])).await {
        Ok(_) => {
            println!("✓ Array of objects: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Array of objects: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 4: Nested array
    println!("\n--- Test 4: Nested array ---");
    match gun.get("test").get("nested_array").put(json!([
        [1, 2, 3],
        [4, 5, 6]
    ])).await {
        Ok(_) => {
            println!("✓ Nested array: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Nested array: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 5: Complex nested structure
    println!("\n--- Test 5: Complex nested structure ---");
    match gun.get("test").get("complex").put(json!({
        "users": [
            {
                "name": "Alice",
                "profile": {
                    "age": 30,
                    "city": "NYC"
                }
            },
            {
                "name": "Bob",
                "profile": {
                    "age": 28,
                    "city": "LA"
                }
            }
        ]
    })).await {
        Ok(_) => {
            println!("✓ Complex nested: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Complex nested: Failed - {}", e);
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

