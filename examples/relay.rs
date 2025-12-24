use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio;

/// Example using a custom relay server
/// Shows that relays are optional - just helpful peers for connectivity
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to relay server...");

    // Option 1: Use your relay server
    let relay_url = "ws://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    let gun = Arc::new(Gun::with_options(GunOptions::with_relay(relay_url)).await?);

    // Option 2: Use multiple relays for redundancy
    // let gun = Arc::new(Gun::with_options(GunOptions::with_peers(vec![
    //     "ws://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun".to_string(),
    //     "ws://backup-relay.com/gun".to_string(),
    // ])));

    // Option 3: Run fully P2P without any relay
    // let gun = Arc::new(Gun::new());

    // Test: Save some data
    println!("Saving data...");
    gun.get("test").get("message").put(json!("Hello from Rust!")).await?;

    // Test: Read data
    println!("Reading data...");
    gun.get("test").get("message").once(|data, key| {
        println!("Received: {:?}", data);
    }).await?;

    println!("Relay example complete!");
    println!("\nNote: The relay server is just a helpful peer - Gun.js is fully decentralized!");
    println!("You can also run without any relay for pure P2P networking.");

    Ok(())
}

