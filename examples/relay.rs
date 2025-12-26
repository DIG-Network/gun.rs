use gun::{Gun, GunOptions};
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;

/// Example using a custom relay server
/// Shows that relays are optional - just helpful peers for connectivity
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to relay server...");

    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();

    // Option 1: Use your relay server
    let relay_url = "ws://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    let options = GunOptions {
        peers: vec![relay_url.to_string()],
        ..Default::default()
    };
    let gun = Arc::new(Gun::with_options(secret_key, public_key, options).await?);

    // Option 2: Use multiple relays for redundancy
    // let secret_key = SecretKey::from_seed(&[0u8; 32]);
    // let public_key = secret_key.public_key();
    // let options = GunOptions {
    //     peers: vec![
    //         "ws://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun".to_string(),
    //         "ws://backup-relay.com/gun".to_string(),
    //     ],
    //     ..Default::default()
    // };
    // let gun = Arc::new(Gun::with_options(secret_key, public_key, options).await?);

    // Option 3: Run fully P2P without any relay
    // let secret_key = SecretKey::from_seed(&[0u8; 32]);
    // let public_key = secret_key.public_key();
    // let gun = Arc::new(Gun::new(secret_key, public_key));

    // Test: Save some data
    println!("Saving data...");
    gun.get("test")
        .get("message")
        .put(json!("Hello from Rust!"))
        .await?;

    // Test: Read data
    println!("Reading data...");
    gun.get("test")
        .get("message")
        .once(|data, _key| {
            println!("Received: {:?}", data);
        })
        .await?;

    println!("Relay example complete!");
    println!("\nNote: The relay server is just a helpful peer - Gun.js is fully decentralized!");
    println!("You can also run without any relay for pure P2P networking.");

    Ok(())
}
