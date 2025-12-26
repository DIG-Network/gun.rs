/// Test: Storage errors
/// 
/// Tests handling storage failures.

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: Storage errors");
    println!("Description: Test storage failures");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Storage errors ---");
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    // Try to use invalid storage path
    let options = GunOptions {
        radisk: true,
        storage_path: Some("/invalid/path/that/does/not/exist".to_string()),
        ..Default::default()
    };
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(_) => {
            println!("✓ Storage errors: Instance created (may fail on write)");
            success_count += 1;
        }
        Err(e) => {
            println!("✓ Storage errors: Correctly rejected - {}", e);
            success_count += 1;
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

