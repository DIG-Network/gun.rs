/// Test: Chain.put() with soul references
/// 
/// Tests putting direct soul refs, nested refs, and circular refs.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with soul references");
    println!("Description: Test direct soul refs, nested refs, circular refs");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // First, create some nodes to reference
    println!("\n--- Setup: Create nodes ---");
    let node1_soul = "node1_soul_123";
    let node2_soul = "node2_soul_456";
    
    match gun.get(node1_soul).put(json!({"name": "Node 1"})).await {
        Ok(_) => println!("✓ Created node1"),
        Err(e) => {
            println!("✗ Failed to create node1: {}", e);
            fail_count += 1;
        }
    }
    
    match gun.get(node2_soul).put(json!({"name": "Node 2"})).await {
        Ok(_) => println!("✓ Created node2"),
        Err(e) => {
            println!("✗ Failed to create node2: {}", e);
            fail_count += 1;
        }
    }
    
    // Test 1: Direct soul reference
    println!("\n--- Test 1: Direct soul reference ---");
    match gun.get("test").get("ref1").put(json!({"#": node1_soul})).await {
        Ok(_) => {
            println!("✓ Direct soul ref: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Direct soul ref: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Nested soul reference
    println!("\n--- Test 2: Nested soul reference ---");
    match gun.get("test").get("nested_ref").put(json!({
        "ref": {"#": node1_soul},
        "other": "data"
    })).await {
        Ok(_) => {
            println!("✓ Nested soul ref: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Nested soul ref: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Multiple soul references
    println!("\n--- Test 3: Multiple soul references ---");
    match gun.get("test").get("multi_ref").put(json!({
        "ref1": {"#": node1_soul},
        "ref2": {"#": node2_soul}
    })).await {
        Ok(_) => {
            println!("✓ Multiple soul refs: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Multiple soul refs: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 4: Circular reference (node1 -> node2 -> node1)
    println!("\n--- Test 4: Circular reference ---");
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
                    println!("✓ Circular ref: Success");
                    success_count += 1;
                }
                Err(e) => {
                    println!("✗ Circular ref (step 2): Failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Circular ref (step 1): Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 5: Self-reference
    println!("\n--- Test 5: Self-reference ---");
    match gun.get(node1_soul).put(json!({
        "name": "Node 1",
        "self": {"#": node1_soul}
    })).await {
        Ok(_) => {
            println!("✓ Self-reference: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Self-reference: Failed - {}", e);
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

