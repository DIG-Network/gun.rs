/// Test: Gun::with_options() with various option combinations
/// 
/// Tests creating Gun instances with different combinations of options.

use gun::{Gun, GunOptions, WebRTCOptions};
use tempfile::TempDir;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with option combinations");
    println!("Description: Test various combinations of options");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Storage + peers
    println!("\n--- Test 1: Storage + peers ---");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        localStorage: true,
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key1 = SecretKey::from_seed(&[1 u8; 32]);
    let public_key1 = secret_key1.public_key();
    match Gun::with_options(secret_key1, public_key1, options).await {
        Ok(_) => {
            println!("✓ Storage + peers: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Storage + peers: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Storage + WebRTC
    println!("\n--- Test 2: Storage + WebRTC ---");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options = GunOptions {
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        localStorage: true,
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
            println!("✓ Storage + WebRTC: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Storage + WebRTC: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Peers + WebRTC
    println!("\n--- Test 3: Peers + WebRTC ---");
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        webrtc: WebRTCOptions {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key3 = SecretKey::from_seed(&[3 u8; 32]);
    let public_key3 = secret_key3.public_key();
    match Gun::with_options(secret_key3, public_key3, options).await {
        Ok(_) => {
            println!("✓ Peers + WebRTC: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Peers + WebRTC: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 4: All options combined
    println!("\n--- Test 4: All options combined ---");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        localStorage: true,
        radisk: true,
        super_peer: false,
        port: None,
        webrtc: WebRTCOptions {
            enabled: true,
            ..Default::default()
        },
    };
    // Generate BLS key pair
    let secret_key4 = SecretKey::from_seed(&[4 u8; 32]);
    let public_key4 = secret_key4.public_key();
    match Gun::with_options(secret_key4, public_key4, options).await {
        Ok(_) => {
            println!("✓ All options: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ All options: Failed - {}", e);
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

