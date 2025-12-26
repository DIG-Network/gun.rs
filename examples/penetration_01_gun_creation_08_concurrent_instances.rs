/// Test: Multiple Gun instances created concurrently
/// 
/// Tests creating multiple Gun instances simultaneously.

use gun::Gun;
use chia_bls::{SecretKey, PublicKey};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Multiple Gun instances created concurrently");
    println!("Description: Create 10 instances simultaneously");
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let secret_key = SecretKey::from_seed(&[i as u8; 32]);
            let public_key = secret_key.public_key();
            match std::panic::catch_unwind(|| {
                Gun::new(secret_key, public_key)
            }) {
                Ok(gun) => {
                    println!("✓ Instance {}: Created successfully", i);
                    Ok(gun)
                }
                Err(e) => {
                    println!("✗ Instance {}: Panic - {:?}", i, e);
                    Err(format!("Panic in instance {}", i))
                }
            }
        });
        handles.push(handle);
    }
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => {
                println!("Error: {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("Join error: {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    // Give time for any async operations
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

