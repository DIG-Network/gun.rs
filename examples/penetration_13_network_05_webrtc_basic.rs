/// Test: WebRTC peer connection
/// 
/// Tests WebRTC peer connection.

use gun::{Gun, GunOptions, WebRTCOptions};

#[tokio::main]
async fn main() {
    println!("Test: WebRTC peer connection");
    println!("Description: Test WebRTC peer connection");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: WebRTC peer connection ---");
    let options = GunOptions {
        webrtc: WebRTCOptions {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };
    
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ WebRTC peer connection: Instance created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ WebRTC peer connection: Failed - {}", e);
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

