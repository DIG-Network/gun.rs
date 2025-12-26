/// Test: SEA.sign() and SEA.verify() - Sign and verify
/// 
/// Tests signing data and verifying signatures.

use gun::sea::{pair, sign, verify};
use serde_json::json;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.sign() and SEA.verify() - Sign and verify");
    println!("Description: Sign data and verify signature");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Sign and verify
    println!("\n--- Test: Sign and verify ---");
    match pair().await {
        Ok(keypair) => {
            let data = json!({"message": "hello world"});
            match sign(&data, &keypair).await {
                Ok(signed) => {
                    match verify(&signed, &keypair.pub_key).await {
                        Ok(verified) => {
                            if verified == data {
                                println!("✓ Sign and verify: Success");
                                success_count += 1;
                            } else {
                                println!("✗ Sign and verify: Verified data doesn't match");
                                fail_count += 1;
                            }
                        }
                        Err(e) => {
                            println!("✗ Sign and verify: Verification failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Sign and verify: Sign failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Sign and verify: Pair generation failed - {}", e);
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

