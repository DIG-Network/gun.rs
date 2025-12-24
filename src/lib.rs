pub mod chain;
pub mod core;
pub mod dam;
pub mod dup;
pub mod error;
pub mod events;
pub mod graph;
pub mod gun;
pub mod network;
pub mod sea;
pub mod state;
pub mod storage;
pub mod valid;
pub mod websocket;

pub use chain::Chain;
pub use error::GunError;
pub use gun::{Gun, GunOptions};
pub use valid::valid;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_put_get() {
        let gun = Gun::new();
        let chain = gun.get("test");
        chain
            .put(serde_json::json!({"name": "test"}))
            .await
            .unwrap();

        let mut called = false;
        chain
            .once(|data, _key| {
                called = true;
                assert!(data.is_object());
            })
            .await
            .unwrap();
        assert!(called);
    }
}
