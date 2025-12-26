/// Test: SEA.encrypt() - Encrypt nested structures
/// 
/// Tests encrypting nested structures.

use gun::sea::{pair, encrypt, decrypt};
use serde_json::json;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.encrypt() - Encrypt nested structures");
    println!("Description: Encrypt nested structures");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair) => {
            let nested = json!({
                "level1": {
                    "level2": {
                        "level3": "deep value",
                        "array": [1, 2, 3]
                    }
                }
            });
            
            match encrypt(&nested, &keypair, None).await {
                Ok(encrypted) => {
                    match decrypt(&encrypted, &keypair, None).await {
                            Ok(decrypted) => {
                                if decrypted == nested {
                                    println!("✓ Encrypt nested: Success");
                                    success_count += 1;
                                } else {
                                    println!("✗ Encrypt nested: Decrypted data doesn't match");
                                    fail_count += 1;
                                }
                            }
                            Err(e) => {
                                println!("✗ Encrypt nested: Decrypt failed - {}", e);
                                fail_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Encrypt nested: Encrypt failed - {}", e);
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

