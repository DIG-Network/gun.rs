/// Test: SEA.verify() - Verify with wrong key
/// 
/// Tests verifying with the wrong public key (should fail).

use gun::sea::{pair, sign, verify};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: SEA.verify() - Verify with wrong key");
    println!("Description: Verify signature with wrong public key (should fail)");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Sign with one key, verify with another
    println!("\n--- Test: Verify with wrong key ---");
    match pair().await {
        Ok(keypair1) => {
            match pair().await {
                Ok(keypair2) => {
                    let data = json!({"message": "test"});
                    match sign(&data, &keypair1).await {
                        Ok(signed) => {
                            match verify(&signed, &keypair2.pub_key).await {
                                Ok(_) => {
                                    println!("✗ Wrong key: Verification succeeded (should have failed)");
                                    fail_count += 1;
                                }
                                Err(_) => {
                                    println!("✓ Wrong key: Verification correctly failed");
                                    success_count += 1;
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ Wrong key: Sign failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Wrong key: Pair2 generation failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Wrong key: Pair1 generation failed - {}", e);
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

