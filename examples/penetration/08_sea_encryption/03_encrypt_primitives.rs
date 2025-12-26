/// Test: SEA.encrypt() - Encrypt all data types
/// 
/// Tests encrypting strings, numbers, booleans, objects, and arrays.

use gun::sea::{pair, encrypt, decrypt};

#[tokio::main]
async fn main() {
    println!("Test: SEA.encrypt() - Encrypt all data types");
    println!("Description: Encrypt strings, numbers, booleans, objects, arrays");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair) => {
            // Test string
            let test_string = serde_json::json!("hello");
            if test_encrypt_decrypt(&test_string, &keypair, "string").await {
                success_count += 1;
            } else {
                fail_count += 1;
            }
            
            // Test number
            let test_number = serde_json::json!(42);
            if test_encrypt_decrypt(&test_number, &keypair, "number").await {
                success_count += 1;
            } else {
                fail_count += 1;
            }
            
            // Test boolean
            let test_bool = serde_json::json!(true);
            if test_encrypt_decrypt(&test_bool, &keypair, "boolean").await {
                success_count += 1;
            } else {
                fail_count += 1;
            }
            
            // Test object
            let test_obj = serde_json::json!({"key": "value"});
            if test_encrypt_decrypt(&test_obj, &keypair, "object").await {
                success_count += 1;
            } else {
                fail_count += 1;
            }
            
            // Test array
            let test_arr = serde_json::json!([1, 2, 3]);
            if test_encrypt_decrypt(&test_arr, &keypair, "array").await {
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Pair generation failed - {}", e);
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

async fn test_encrypt_decrypt(data: &serde_json::Value, keypair: &gun::sea::KeyPair, name: &str) -> bool {
    println!("\n--- Test: Encrypt {} ---", name);
    match encrypt(data, keypair, None).await {
        Ok(encrypted) => {
            match decrypt(&encrypted, keypair, None).await {
                Ok(decrypted) => {
                    if decrypted == *data {
                        println!("✓ Encrypt {}: Success", name);
                        true
                    } else {
                        println!("✗ Encrypt {}: Decrypted data doesn't match", name);
                        false
                    }
                }
                Err(e) => {
                    println!("✗ Encrypt {}: Decrypt failed - {}", name, e);
                    false
                }
            }
        }
        Err(e) => {
            println!("✗ Encrypt {}: Encrypt failed - {}", name, e);
            false
        }
    }
}

