/// Test: Gun::with_options() with various option combinations
/// 
/// Tests creating Gun instances with different combinations of options.

use gun::{Gun, GunOptions, WebRTCOptions};
use tempfile::TempDir;

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
    match Gun::with_options(options).await {
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
    match Gun::with_options(options).await {
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
    match Gun::with_options(options).await {
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
    match Gun::with_options(options).await {
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

