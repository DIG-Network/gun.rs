/// Test: SEA.pair() - Generate single key pair
/// 
/// Tests generating a single key pair.

use gun::sea::pair;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: SEA.pair() - Generate single key pair");
    println!("Description: Generate a single key pair");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Generate key pair
    println!("\n--- Test: Generate key pair ---");
    match pair().await {
        Ok(keypair) => {
            println!("✓ Key pair generated: Success");
            println!("  - pub_key: {}...", &keypair.pub_key[..20.min(keypair.pub_key.len())]);
            println!("  - priv_key: {}...", &keypair.priv_key[..20.min(keypair.priv_key.len())]);
            if let Some(ref epub) = keypair.epub_key {
                println!("  - epub_key: {}...", &epub[..20.min(epub.len())]);
            }
            if let Some(ref epriv) = keypair.epriv_key {
                println!("  - epriv_key: {}...", &epriv[..20.min(epriv.len())]);
            }
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Key pair generation: Failed - {}", e);
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

