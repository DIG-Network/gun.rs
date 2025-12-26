/// Test: SledStorage operations
/// 
/// Tests SledStorage basic operations.

use gun::{Gun, GunOptions};
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    println!("Test: SledStorage operations");
    println!("Description: Test SledStorage basic operations");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: SledStorage ---");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options = GunOptions {
        radisk: true,
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        ..Default::default()
    };
    
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ SledStorage: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ SledStorage: Failed - {}", e);
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

