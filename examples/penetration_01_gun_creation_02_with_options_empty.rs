/// Test: Gun::with_options() with empty/default options
/// 
/// Tests creating a Gun instance with empty GunOptions (all defaults).
/// Should behave similarly to Gun::new().

use gun::{Gun, GunOptions};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with empty options");
    println!("Description: Create Gun instance with default GunOptions");
    
    let options = GunOptions::default();
    
    match Gun::with_options(options).await {
        Ok(gun) => {
            println!("✓ Success: Gun instance created with empty options");
            println!("  - Instance type: Gun");
            println!("  - Options: All defaults");
            std::process::exit(0);
        }
        Err(e) => {
            println!("✗ Error: Failed to create Gun instance");
            println!("  Error: {}", e);
            std::process::exit(1);
        }
    }
}

