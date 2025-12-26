/// Test: Switch between storage backends
/// 
/// Tests switching between different storage backends.

use gun::{Gun, GunOptions};
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    println!("Test: Switch between storage backends");
    println!("Description: Switch between different storage backends");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Create instances with different storage backends
    println!("\n--- Test: Switch storage backends ---");
    
    // MemoryStorage
    let options1 = GunOptions {
        localStorage: false,
        storage_path: None,
        ..Default::default()
    };
    if Gun::with_options(options1).await.is_ok() {
        success_count += 1;
    } else {
        fail_count += 1;
    }
    
    // SledStorage
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options2 = GunOptions {
        radisk: true,
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        ..Default::default()
    };
    if Gun::with_options(options2).await.is_ok() {
        success_count += 1;
    } else {
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

