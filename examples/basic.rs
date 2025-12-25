use gun::Gun;
use serde_json::json;
use std::sync::Arc;

/// Basic example matching the JavaScript Gun.js quickstart
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Gun instance
    let gun = Arc::new(Gun::new());

    // Save data - equivalent to:
    // gun.get('mark').put({name: "Mark", email: "mark@gun.eco"})
    let mark_chain = gun.get("mark");
    mark_chain
        .put(json!({
            "name": "Mark",
            "email": "mark@gun.eco"
        }))
        .await?;

    // Read data - equivalent to:
    // gun.get('mark').on((data, key) => { console.log("realtime updates:", data) })
    let mark_chain = gun.get("mark");
    mark_chain.on(|data, _key| {
        println!("realtime updates: {:?}", data);
    });

    // Live updates - equivalent to:
    // setInterval(() => { gun.get('mark').get('live').put(Math.random()) }, 9)
    for i in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(9)).await;
        gun.get("mark").get("live").put(json!(i as f64)).await?;
    }

    Ok(())
}
