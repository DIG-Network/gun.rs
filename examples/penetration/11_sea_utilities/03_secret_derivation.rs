/// Test: SEA.secret() - ECDH secret derivation
/// 
/// Tests ECDH secret derivation.

use gun::sea::{pair, secret};

#[tokio::main]
async fn main() {
    println!("Test: SEA.secret() - ECDH secret derivation");
    println!("Description: Test ECDH secret derivation");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(pair1) => {
            match pair().await {
                Ok(pair2) => {
                    if let (Some(ref epub1), Some(ref epriv1)) = (pair1.epub_key.as_ref(), pair1.epriv_key.as_ref()) {
                        if let (Some(ref epub2), Some(ref epriv2)) = (pair2.epub_key.as_ref(), pair2.epriv_key.as_ref()) {
                            println!("\n--- Test: ECDH secret derivation ---");
                            match secret(epub1, epriv2, epub2).await {
                                Ok(secret1) => {
                                    match secret(epub2, epriv1, epub1).await {
                                        Ok(secret2) => {
                                            if secret1 == secret2 {
                                                println!("✓ Secret derivation: Success (secrets match)");
                                                success_count += 1;
                                            } else {
                                                println!("✗ Secret derivation: Secrets don't match");
                                                fail_count += 1;
                                            }
                                        }
                                        Err(e) => {
                                            println!("✗ Secret derivation: Second secret failed - {}", e);
                                            fail_count += 1;
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("✗ Secret derivation: First secret failed - {}", e);
                                    fail_count += 1;
                                }
                            }
                        } else {
                            println!("✗ Missing pair2 encryption keys");
                            fail_count += 1;
                        }
                    } else {
                        println!("✗ Missing pair1 encryption keys");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ Pair2 generation failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Pair1 generation failed - {}", e);
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

