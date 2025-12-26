/// Test: LocalStorage operations
/// 
/// Tests LocalStorage basic operations.

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    println!("Test: LocalStorage operations");
    println!("Description: Test LocalStorage basic operations");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: LocalStorage ---");
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options = GunOptions {
        localStorage: true,
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        radisk: false,
        ..Default::default()
    };
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(_) => {
            println!("✓ LocalStorage: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ LocalStorage: Failed - {}", e);
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

