/// Test: SEA.sign() - Sign primitives
/// 
/// Tests signing strings, numbers, objects, and arrays.

use gun::sea::{pair, sign, verify};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.sign() - Sign primitives");
    println!("Description: Sign strings, numbers, objects, arrays");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair) => {
            // Test 1: Sign string
            println!("\n--- Test 1: Sign string ---");
            match sign(&serde_json::json!("hello"), &keypair).await {
                Ok(signed) => {
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(_) => {
                            println!("✓ Sign string: Success");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Sign string: Verify failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Sign string: Failed - {}", e);
                    fail_count += 1;
                }
            }
            
            // Test 2: Sign number
            println!("\n--- Test 2: Sign number ---");
            match sign(&serde_json::json!(42), &keypair).await {
                Ok(signed) => {
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(_) => {
                            println!("✓ Sign number: Success");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Sign number: Verify failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Sign number: Failed - {}", e);
                    fail_count += 1;
                }
            }
            
            // Test 3: Sign object
            println!("\n--- Test 3: Sign object ---");
            match sign(&serde_json::json!({"key": "value"}), &keypair).await {
                Ok(signed) => {
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(_) => {
                            println!("✓ Sign object: Success");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Sign object: Verify failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Sign object: Failed - {}", e);
                    fail_count += 1;
                }
            }
            
            // Test 4: Sign array
            println!("\n--- Test 4: Sign array ---");
            match sign(&serde_json::json!([1, 2, 3]), &keypair).await {
                Ok(signed) => {
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(_) => {
                            println!("✓ Sign array: Success");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Sign array: Verify failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Sign array: Failed - {}", e);
                    fail_count += 1;
                }
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

