/// Test: SEA.verify() - Verify tampered data
/// 
/// Tests verifying tampered signed data (should fail).

use gun::sea::{pair, sign, verify};
use serde_json::json;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.verify() - Verify tampered data");
    println!("Description: Verify tampered signed data (should fail)");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Sign data, tamper with it, verify (should fail)
    println!("\n--- Test: Verify tampered data ---");
    match pair().await {
        Ok(keypair) => {
            let data = json!({"message": "original"});
            match sign(&data, &keypair).await {
                Ok(mut signed) => {
                    // Tamper with the signed data
                    if let Some(obj) = signed.as_object_mut() {
                        if let Some(sig) = obj.get_mut("sig") {
                            if let Some(sig_str) = sig.as_str() {
                                let mut tampered = sig_str.to_string();
                                tampered.push('X'); // Tamper with signature
                                *sig = json!(tampered);
                            }
                        }
                    }
                    
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(_) => {
                            println!("✗ Tampered: Verification succeeded (should have failed)");
                            fail_count += 1;
                        }
                        Err(_) => {
                            println!("✓ Tampered: Verification correctly failed");
                            success_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Tampered: Sign failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Tampered: Pair generation failed - {}", e);
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

