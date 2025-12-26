/// Test: SEA.pair() - Generate multiple pairs
/// 
/// Tests generating multiple key pairs.

use gun::sea::pair;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.pair() - Generate multiple pairs");
    println!("Description: Generate multiple key pairs");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Generate 10 key pairs
    println!("\n--- Test: Generate 10 key pairs ---");
    let mut generated = 0;
    for i in 0..10 {
        match pair().await {
            Ok(keypair) => {
                println!("  Pair {}: Generated", i + 1);
                generated += 1;
            }
            Err(e) => {
                println!("  Pair {}: Failed - {}", i + 1, e);
                fail_count += 1;
            }
        }
    }
    
    if generated == 10 {
        println!("✓ Multiple pairs: All 10 generated");
        success_count += 1;
    } else {
        println!("✗ Multiple pairs: Only {}/10 generated", generated);
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

