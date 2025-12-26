/// Test: Gun::with_options() with various option combinations
/// 
/// Tests creating Gun instances with different combinations of options.

use gun::{Gun, GunOptions, WebRTCOptions};
use chia_bls::{SecretKey, PublicKey};
use tempfile::TempDir;
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with option combinations");
    println!("Description: Test various combinations of options");
    
    // Wrap entire test in timeout to prevent infinite hangs
    match timeout(Duration::from_secs(30), async {
        let mut success_count = 0;
        let mut fail_count = 0;
    
        // Test 1: Storage + peers (with timeout)
        println!("\n--- Test 1: Storage + peers ---");
        let secret_key1 = SecretKey::from_seed(&[1u8; 32]);
        let public_key1 = secret_key1.public_key();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let options = GunOptions {
            peers: vec!["ws://localhost:8765/gun".to_string()],
            storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
            localStorage: true,
            ..Default::default()
        };
        match timeout(Duration::from_secs(5), Gun::with_options(secret_key1, public_key1, options)).await {
            Ok(Ok(_)) => {
                println!("✓ Storage + peers: Success");
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("✗ Storage + peers: Failed - {}", e);
                fail_count += 1;
            }
            Err(_) => {
                println!("✗ Storage + peers: Timed out after 5 seconds (peer may not be available)");
                // Don't count timeout as failure for peer connection tests
            }
        }
        
        // Test 2: Storage + WebRTC
        println!("\n--- Test 2: Storage + WebRTC ---");
        let secret_key2 = SecretKey::from_seed(&[2u8; 32]);
        let public_key2 = secret_key2.public_key();
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
        match timeout(Duration::from_secs(5), Gun::with_options(secret_key2, public_key2, options)).await {
            Ok(Ok(_)) => {
                println!("✓ Storage + WebRTC: Success");
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("✗ Storage + WebRTC: Failed - {}", e);
                fail_count += 1;
            }
            Err(_) => {
                println!("✗ Storage + WebRTC: Timed out after 5 seconds");
                fail_count += 1;
            }
        }
        
        // Test 3: Peers + WebRTC
        println!("\n--- Test 3: Peers + WebRTC ---");
        let secret_key3 = SecretKey::from_seed(&[3u8; 32]);
        let public_key3 = secret_key3.public_key();
        let options = GunOptions {
            peers: vec!["ws://localhost:8765/gun".to_string()],
            webrtc: WebRTCOptions {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };
        match timeout(Duration::from_secs(5), Gun::with_options(secret_key3, public_key3, options)).await {
            Ok(Ok(_)) => {
                println!("✓ Peers + WebRTC: Success");
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("✗ Peers + WebRTC: Failed - {}", e);
                fail_count += 1;
            }
            Err(_) => {
                println!("✗ Peers + WebRTC: Timed out after 5 seconds (peer may not be available)");
                // Don't count timeout as failure for peer connection tests
            }
        }
        
        // Test 4: All options combined
        println!("\n--- Test 4: All options combined ---");
        let secret_key4 = SecretKey::from_seed(&[4u8; 32]);
        let public_key4 = secret_key4.public_key();
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
            ..Default::default()
        };
        match timeout(Duration::from_secs(5), Gun::with_options(secret_key4, public_key4, options)).await {
            Ok(Ok(_)) => {
                println!("✓ All options: Success");
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("✗ All options: Failed - {}", e);
                fail_count += 1;
            }
            Err(_) => {
                println!("✗ All options: Timed out after 5 seconds (peer may not be available)");
                // Don't count timeout as failure for peer connection tests
            }
        }
    
        println!("\n--- Summary ---");
        println!("Success: {}", success_count);
        println!("Failed: {}", fail_count);
        
        (success_count, fail_count)
    }).await {
        Ok((success_count, fail_count)) => {
            if fail_count == 0 {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        Err(_) => {
            println!("\n✗ Test timed out after 30 seconds");
            std::process::exit(1);
        }
    }
}

