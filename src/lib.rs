pub mod chain;
pub mod core;
pub mod dam;
pub mod dup;
pub mod error;
pub mod events;
pub mod graph;
pub mod gun;
pub mod sea;
pub mod state;
pub mod storage;
pub mod valid;
pub mod webrtc;
pub mod websocket;

pub use chain::Chain;
pub use error::GunError;
pub use gun::{Gun, GunOptions};
pub use sea::*;
pub use valid::valid;
pub use valid::{is_valid_data, valid_soul};
pub use webrtc::{WebRTCManager, WebRTCOptions, WebRTCPeer};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_put_get() {
        let gun = Gun::new();
        let chain = gun.get("test");
        // In tests, unwrap is acceptable for error handling
        chain
            .put(serde_json::json!({"name": "test"}))
            .await
            .expect("put should succeed in test");

        let mut called = false;
        chain
            .once(|data, _key| {
                called = true;
                assert!(data.is_object());
            })
            .await
            .expect("once should succeed in test");
        assert!(called);
    }
}
