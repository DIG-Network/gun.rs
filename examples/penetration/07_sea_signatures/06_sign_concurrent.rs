/// Test: SEA.sign() - Concurrent signing
/// 
/// Tests concurrent signing operations.

use gun::sea::{pair, sign, verify};
use serde_json::json;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.sign() - Concurrent signing");
    println!("Description: Test concurrent signing operations");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(keypair) => {
            // Test: Concurrent signing
            println!("\n--- Test: Concurrent signing (10 operations) ---");
            let mut handles = vec![];
            for i in 0..10 {
                let keypair_clone = keypair.clone();
                let data = json!({"id": i});
                let handle = tokio::spawn(async move {
                    sign(&data, &keypair_clone).await
                });
                handles.push(handle);
            }
            
            let mut concurrent_success = 0;
            for handle in handles {
                match handle.await {
                    Ok(Ok(_)) => concurrent_success += 1,
                    Ok(Err(e)) => {
                        println!("✗ Concurrent signing: Error - {}", e);
                        fail_count += 1;
                    }
                    Err(e) => {
                        println!("✗ Concurrent signing: Join error - {:?}", e);
                        fail_count += 1;
                    }
                }
            }
            
            if concurrent_success == 10 {
                println!("✓ Concurrent signing: All 10 succeeded");
                success_count += 1;
            } else {
                println!("✗ Concurrent signing: Only {}/10 succeeded", concurrent_success);
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

