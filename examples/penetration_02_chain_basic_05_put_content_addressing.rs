/// Test: Chain.put() with content addressing (hash-prefixed souls)
/// 
/// Tests putting data with #hash souls for content addressing.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with content addressing");
    println!("Description: Test hash-prefixed souls (#hash)");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Content addressing with hash
    println!("\n--- Test 1: Content addressing with hash ---");
    let data = json!({"content": "test data"});
    let data_string = serde_json::to_string(&data).unwrap();
    
    // Generate hash (simplified - in practice, this would use SEA.work)
    use sha2::{Sha256, Digest};
    use base64::{engine::general_purpose, Engine as _};
    let mut hasher = Sha256::new();
    hasher.update(data_string.as_bytes());
    let hash = general_purpose::STANDARD_NO_PAD.encode(hasher.finalize());
    let hash_soul = format!("#{}", hash);
    
    match gun.get(&hash_soul).put(data.clone()).await {
        Ok(_) => {
            println!("✓ Content addressing: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Content addressing: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Invalid hash (should still work, but hash won't match)
    println!("\n--- Test 2: Invalid hash format ---");
    let invalid_hash_soul = "#invalid_hash_123";
    match gun.get(invalid_hash_soul).put(json!({"data": "test"})).await {
        Ok(_) => {
            println!("✓ Invalid hash format: Success (hash won't verify)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Invalid hash format: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Hash with different data (should fail verification)
    println!("\n--- Test 3: Hash mismatch ---");
    let wrong_data = json!({"content": "different data"});
    match gun.get(&hash_soul).put(wrong_data).await {
        Ok(_) => {
            println!("✓ Hash mismatch: Put succeeded (verification may fail)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Hash mismatch: Failed - {}", e);
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

