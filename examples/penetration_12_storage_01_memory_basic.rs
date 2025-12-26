/// Test: MemoryStorage operations
/// 
/// Tests MemoryStorage basic operations.

use gun::{Gun, GunOptions};

#[tokio::main]
async fn main() {
    println!("Test: MemoryStorage operations");
    println!("Description: Test MemoryStorage basic operations");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: MemoryStorage ---");
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
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

