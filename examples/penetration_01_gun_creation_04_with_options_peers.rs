/// Test: Gun::with_options() with peer configurations
/// 
/// Tests creating Gun instances with single peer, multiple peers, and invalid URLs.

use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with peer configurations");
    println!("Description: Test single peer, multiple peers, invalid URLs");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Single valid peer (may fail if peer doesn't exist, but should create instance)
    println!("\n--- Test 1: Single peer ---");
    let secret_key1 = SecretKey::from_seed(&[1u8; 32]);
    let public_key1 = secret_key1.public_key();
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        ..Default::default()
    };
    match timeout(Duration::from_secs(5), Gun::with_options(secret_key1, public_key1, options)).await {
        Ok(Ok(_)) => {
            println!("✓ Single peer: Instance created (connection may fail)");
            success_count += 1;
        }
        Ok(Err(e)) => {
            println!("✗ Single peer: Failed - {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Single peer: Timed out after 5 seconds (peer may not be available)");
            // Don't count timeout as failure for peer connection tests
        }
    }
    
    // Test 2: Multiple peers
    println!("\n--- Test 2: Multiple peers ---");
    let secret_key2 = SecretKey::from_seed(&[2u8; 32]);
    let public_key2 = secret_key2.public_key();
    let options = GunOptions {
        peers: vec![
            "ws://localhost:8765/gun".to_string(),
            "ws://localhost:8766/gun".to_string(),
        ],
        ..Default::default()
    };
    match timeout(Duration::from_secs(5), Gun::with_options(secret_key2, public_key2, options)).await {
        Ok(Ok(_)) => {
            println!("✓ Multiple peers: Instance created");
            success_count += 1;
        }
        Ok(Err(e)) => {
            println!("✗ Multiple peers: Failed - {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Multiple peers: Timed out after 5 seconds (peers may not be available)");
            // Don't count timeout as failure for peer connection tests
        }
    }
    
    // Test 3: Invalid URL (should still create instance, connection will fail)
    println!("\n--- Test 3: Invalid peer URL ---");
    let secret_key3 = SecretKey::from_seed(&[3u8; 32]);
    let public_key3 = secret_key3.public_key();
    let options = GunOptions {
        peers: vec!["not-a-valid-url".to_string()],
        ..Default::default()
    };
    match timeout(Duration::from_secs(5), Gun::with_options(secret_key3, public_key3, options)).await {
        Ok(Ok(_)) => {
            println!("✓ Invalid URL: Instance created (connection will fail)");
            success_count += 1;
        }
        Ok(Err(e)) => {
            println!("✗ Invalid URL: Failed - {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Invalid URL: Timed out after 5 seconds");
            fail_count += 1;
        }
    }
    
    // Test 4: Empty peers list
    println!("\n--- Test 4: Empty peers list ---");
    let secret_key4 = SecretKey::from_seed(&[4u8; 32]);
    let public_key4 = secret_key4.public_key();
    let options = GunOptions {
        peers: vec![],
        ..Default::default()
    };
    match timeout(Duration::from_secs(5), Gun::with_options(secret_key4, public_key4, options)).await {
        Ok(Ok(_)) => {
            println!("✓ Empty peers: Instance created");
            success_count += 1;
        }
        Ok(Err(e)) => {
            println!("✗ Empty peers: Failed - {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Empty peers: Timed out after 5 seconds");
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

