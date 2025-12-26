/// Test: SEA.encrypt() and SEA.decrypt() - Peer-to-peer encryption
/// 
/// Tests encrypting with peer's public key and decrypting with own private key.

use gun::sea::{pair, encrypt, decrypt};
use serde_json::json;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.encrypt() and SEA.decrypt() - Peer-to-peer encryption");
    println!("Description: Encrypt with peer's public key, decrypt with own private key");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Generate two key pairs (sender and receiver)
    match pair().await {
        Ok(sender_pair) => {
            match pair().await {
                Ok(receiver_pair) => {
                    if let Some(ref receiver_epub) = receiver_pair.epub_key.as_ref() {
                        // Sender encrypts with receiver's public key
                        let data = json!({"message": "peer-to-peer message"});
                        match encrypt(&data, &sender_pair, Some(receiver_epub)).await {
                            Ok(encrypted) => {
                                // Receiver decrypts with own private key
                                match decrypt(&encrypted, &receiver_pair, Some(receiver_epub)).await {
                                        Ok(decrypted) => {
                                            if decrypted == data {
                                                println!("✓ Peer-to-peer encryption: Success");
                                                success_count += 1;
                                            } else {
                                                println!("✗ Peer-to-peer encryption: Decrypted data doesn't match");
                                                fail_count += 1;
                                            }
                                        }
                                        Err(e) => {
                                            println!("✗ Peer-to-peer encryption: Decrypt failed - {}", e);
                                            fail_count += 1;
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("✗ Peer-to-peer encryption: Encrypt failed - {}", e);
                                    fail_count += 1;
                                }
                            }
                    } else {
                        println!("✗ Peer-to-peer encryption: Missing receiver encryption keys");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ Peer-to-peer encryption: Receiver pair generation failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Peer-to-peer encryption: Sender pair generation failed - {}", e);
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

