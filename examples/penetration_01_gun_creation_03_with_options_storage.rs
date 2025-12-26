/// Test: Gun::with_options() with different storage backends
/// 
/// Tests creating Gun instances with MemoryStorage, SledStorage, and LocalStorage.

use gun::{Gun, GunOptions};
use std::path::Path;
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with different storage backends");
    println!("Description: Test MemoryStorage, SledStorage, and LocalStorage");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: MemoryStorage (no storage_path, localStorage=false)
    println!("\n--- Test 1: MemoryStorage ---");
    let options = GunOptions {
        localStorage: false,
        storage_path: None,
        ..Default::default()
    };
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ MemoryStorage: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ MemoryStorage: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: SledStorage (radisk=true, with storage_path)
    println!("\n--- Test 2: SledStorage ---");
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
    
    // Test 3: LocalStorage (localStorage=true, with storage_path)
    println!("\n--- Test 3: LocalStorage ---");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options = GunOptions {
        localStorage: true,
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        radisk: false,
        ..Default::default()
    };
    match Gun::with_options(options).await {
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

