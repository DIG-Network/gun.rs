use gun::Gun;
use chia_bls::{SecretKey, PublicKey};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Gun.rs server starting...");

    // Generate BLS key pair for the server
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    
    let gun = Arc::new(Gun::new(secret_key, public_key));

    // Example usage
    let chain = gun.get("test");
    chain
        .put(serde_json::json!({
            "name": "Test Node",
            "value": 42
        }))
        .await?;

    println!("Server running...");

    // Keep server running
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    Ok(())
}
