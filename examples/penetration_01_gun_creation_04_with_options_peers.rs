/// Test: Gun::with_options() with peer configurations
/// 
/// Tests creating Gun instances with single peer, multiple peers, and invalid URLs.

use gun::{Gun, GunOptions};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with peer configurations");
    println!("Description: Test single peer, multiple peers, invalid URLs");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Single valid peer (may fail if peer doesn't exist, but should create instance)
    println!("\n--- Test 1: Single peer ---");
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        ..Default::default()
    };
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ Single peer: Instance created (connection may fail)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Single peer: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Multiple peers
    println!("\n--- Test 2: Multiple peers ---");
    let options = GunOptions {
        peers: vec![
            "ws://localhost:8765/gun".to_string(),
            "ws://localhost:8766/gun".to_string(),
        ],
        ..Default::default()
    };
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ Multiple peers: Instance created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Multiple peers: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Invalid URL (should still create instance, connection will fail)
    println!("\n--- Test 3: Invalid peer URL ---");
    let options = GunOptions {
        peers: vec!["not-a-valid-url".to_string()],
        ..Default::default()
    };
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ Invalid URL: Instance created (connection will fail)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Invalid URL: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 4: Empty peers list
    println!("\n--- Test 4: Empty peers list ---");
    let options = GunOptions {
        peers: vec![],
        ..Default::default()
    };
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ Empty peers: Instance created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Empty peers: Failed - {}", e);
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

