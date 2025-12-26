/// Test: SEA.sign() - Sign nested structures
/// 
/// Tests signing nested structures.

use gun::sea::{pair, sign, verify};
use serde_json::json;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.sign() - Sign nested structures");
    println!("Description: Sign nested structures");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair) => {
            // Test: Sign nested object
            println!("\n--- Test: Sign nested object ---");
            let nested = json!({
                "level1": {
                    "level2": {
                        "level3": "deep value"
                    }
                }
            });
            
            match sign(&nested, &keypair).await {
                Ok(signed) => {
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(verified) => {
                            if verified == nested {
                                println!("✓ Sign nested: Success");
                                success_count += 1;
                            } else {
                                println!("✗ Sign nested: Verified data doesn't match");
                                fail_count += 1;
                            }
                        }
                        Err(e) => {
                            println!("✗ Sign nested: Verify failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Sign nested: Failed - {}", e);
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

