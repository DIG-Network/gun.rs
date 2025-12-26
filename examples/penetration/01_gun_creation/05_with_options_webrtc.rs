/// Test: Gun::with_options() with WebRTC configuration
/// 
/// Tests creating Gun instances with WebRTC enabled/disabled.

use gun::{Gun, GunOptions, WebRTCOptions};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with WebRTC configuration");
    println!("Description: Test WebRTC enabled/disabled");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: WebRTC disabled (default)
    println!("\n--- Test 1: WebRTC disabled ---");
    let options = GunOptions {
        webrtc: WebRTCOptions {
            enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key1 = SecretKey::from_seed(&[1 u8; 32]);
    let public_key1 = secret_key1.public_key();
    match Gun::with_options(secret_key1, public_key1, options).await {
        Ok(_) => {
            println!("✓ WebRTC disabled: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ WebRTC disabled: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: WebRTC enabled
    println!("\n--- Test 2: WebRTC enabled ---");
    let options = GunOptions {
        webrtc: WebRTCOptions {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key2 = SecretKey::from_seed(&[2 u8; 32]);
    let public_key2 = secret_key2.public_key();
    match Gun::with_options(secret_key2, public_key2, options).await {
        Ok(_) => {
            println!("✓ WebRTC enabled: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ WebRTC enabled: Failed - {}", e);
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

