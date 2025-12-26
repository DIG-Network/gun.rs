/// Test: SEA.decrypt() - Decrypt with wrong key
/// 
/// Tests decrypting with the wrong key (should fail).

use gun::sea::{pair, encrypt, decrypt};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: SEA.decrypt() - Decrypt with wrong key");
    println!("Description: Decrypt with wrong key (should fail)");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair1) => {
            match pair().await {
                Ok(keypair2) => {
                    let data = json!({"secret": "data"});
                    match encrypt(&data, &keypair1, None).await {
                        Ok(encrypted) => {
                            // Try to decrypt with wrong key
                            match decrypt(&encrypted, &keypair2, None).await {
                                        Ok(_) => {
                                            println!("✗ Wrong key: Decryption succeeded (should have failed)");
                                            fail_count += 1;
                                        }
                                        Err(_) => {
                                            println!("✓ Wrong key: Decryption correctly failed");
                                            success_count += 1;
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("✗ Wrong key: Encrypt failed - {}", e);
                                    fail_count += 1;
                                }
                            }
                }
                Err(e) => {
                    println!("✗ Keypair2 generation failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Keypair1 generation failed - {}", e);
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

