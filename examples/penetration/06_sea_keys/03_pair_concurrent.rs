/// Test: SEA.pair() - Concurrent generation
/// 
/// Tests concurrent key pair generation.

use gun::sea::pair;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.pair() - Concurrent generation");
    println!("Description: Generate key pairs concurrently");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Concurrent generation
    println!("\n--- Test: Concurrent generation (10 pairs) ---");
    let mut handles = vec![];
    for _ in 0..10 {
        let handle = tokio::spawn(async move {
            pair().await
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => concurrent_success += 1,
            Ok(Err(e)) => {
                println!("✗ Concurrent generation: Error - {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("✗ Concurrent generation: Join error - {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_success == 10 {
        println!("✓ Concurrent generation: All 10 succeeded");
        success_count += 1;
    } else {
        println!("✗ Concurrent generation: Only {}/10 succeeded", concurrent_success);
        fail_count += 1;
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

