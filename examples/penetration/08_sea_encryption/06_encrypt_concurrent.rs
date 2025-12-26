/// Test: SEA.encrypt() - Concurrent encryption
/// 
/// Tests concurrent encryption operations.

use gun::sea::{pair, encrypt, decrypt};
use serde_json::json;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.encrypt() - Concurrent encryption");
    println!("Description: Test concurrent encryption operations");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair) => {
            let mut handles = vec![];
            for i in 0..10 {
                let keypair_clone = keypair.clone();
                let data = json!({"id": i});
                let handle = tokio::spawn(async move {
                    encrypt(&data, &keypair_clone, None).await
                });
                handles.push(handle);
            }
                
                let mut concurrent_success = 0;
                for handle in handles {
                    match handle.await {
                        Ok(Ok(_)) => concurrent_success += 1,
                        Ok(Err(e)) => {
                            println!("✗ Concurrent encryption: Error - {}", e);
                            fail_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Concurrent encryption: Join error - {:?}", e);
                            fail_count += 1;
                        }
                    }
                }
                
                if concurrent_success == 10 {
                    println!("✓ Concurrent encryption: All 10 succeeded");
                    success_count += 1;
                } else {
                    println!("✗ Concurrent encryption: Only {}/10 succeeded", concurrent_success);
                    fail_count += 1;
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

