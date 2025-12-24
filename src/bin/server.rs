use gun::Gun;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Gun.rs server starting...");
    
    let gun = Arc::new(Gun::new());
    
    // Example usage
    let chain = gun.get("test");
    chain.put(serde_json::json!({
        "name": "Test Node",
        "value": 42
    })).await?;
    
    println!("Server running...");
    
    // Keep server running
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");
    
    Ok(())
}

