/// Test: Crypto operation failures
/// 
/// Tests handling crypto operation failures.

use gun::sea::{pair, sign, verify};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Crypto operation failures");
    println!("Description: Test crypto operation failures");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Crypto errors ---");
    // Test with invalid key format
    match pair().await {
        Ok(keypair) => {
            // Try to verify with invalid signature
            let invalid_sig = json!({"data": "test", "sig": "invalid_signature"});
            match verify(&invalid_sig, &keypair.pub_key).await {
                Ok(_) => {
                    println!("✗ Crypto errors: Verification succeeded (should have failed)");
                    fail_count += 1;
                }
                Err(_) => {
                    println!("✓ Crypto errors: Verification correctly failed");
                    success_count += 1;
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

