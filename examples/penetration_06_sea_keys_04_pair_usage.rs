/// Test: SEA.pair() - Use generated pairs
/// 
/// Tests using generated key pairs for signing and encryption.

use gun::sea::{pair, sign, verify, encrypt, decrypt};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: SEA.pair() - Use generated pairs");
    println!("Description: Use generated pairs for signing and encryption");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Generate and use for signing
    println!("\n--- Test 1: Use for signing ---");
    match pair().await {
        Ok(keypair) => {
            let data = json!({"message": "test"});
            match sign(&data, &keypair).await {
                Ok(signed) => {
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(verified) => {
                            if verified == data {
                                println!("✓ Signing: Success");
                                success_count += 1;
                            } else {
                                println!("✗ Signing: Verification returned wrong data");
                                fail_count += 1;
                            }
                        }
                        Err(e) => {
                            println!("✗ Signing: Verification failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Signing: Sign failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Signing: Pair generation failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test: Generate and use for encryption
    println!("\n--- Test 2: Use for encryption ---");
    match pair().await {
        Ok(keypair) => {
            let data = json!({"secret": "data"});
            match encrypt(&data, &keypair, None).await {
                Ok(encrypted) => {
                    match decrypt(&encrypted, &keypair, None).await {
                            Ok(decrypted) => {
                                if decrypted == data {
                                    println!("✓ Encryption: Success");
                                    success_count += 1;
                                } else {
                                    println!("✗ Encryption: Decryption returned wrong data");
                                    fail_count += 1;
                                }
                            }
                            Err(e) => {
                                println!("✗ Encryption: Decrypt failed - {}", e);
                                fail_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Encryption: Encrypt failed - {}", e);
                        fail_count += 1;
                    }
                }
        }
        Err(e) => {
            println!("✗ Encryption: Pair generation failed - {}", e);
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

