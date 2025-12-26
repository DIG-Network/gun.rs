/// Test: SEA.encrypt() and SEA.decrypt() - Self-encryption
/// 
/// Tests encrypting and decrypting data with own keys.

use gun::sea::{pair, encrypt, decrypt};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: SEA.encrypt() and SEA.decrypt() - Self-encryption");
    println!("Description: Encrypt and decrypt with own keys");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair) => {
            // Self-encryption: use None for their_epub
            let data = json!({"secret": "self-encrypted data"});
use chia_bls::{SecretKey, PublicKey};
            match encrypt(&data, &keypair, None).await {
                Ok(encrypted) => {
                    match decrypt(&encrypted, &keypair, None).await {
                            Ok(decrypted) => {
                                if decrypted == data {
                                    println!("✓ Self-encryption: Success");
                                    success_count += 1;
                                } else {
                                    println!("✗ Self-encryption: Decrypted data doesn't match");
                                    fail_count += 1;
                                }
                            }
                            Err(e) => {
                                println!("✗ Self-encryption: Decrypt failed - {}", e);
                                fail_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Self-encryption: Encrypt failed - {}", e);
                        fail_count += 1;
                    }
                }
        }
        Err(e) => {
            println!("✗ Self-encryption: Pair generation failed - {}", e);
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

