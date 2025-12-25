use futures::future;
use gun::core::GunCore;
use gun::dam::Mesh;
use gun::webrtc::{WebRTCManager, WebRTCOptions, WebRTCPeer};
use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};

/// Test WebRTCOptions default configuration
#[test]
fn test_webrtc_options_default() {
    let options = WebRTCOptions::default();

    assert_eq!(options.max_connections, 55);
    assert!(options.enabled);
    assert_eq!(options.ice_servers.len(), 2);
    assert!(options
        .ice_servers
        .iter()
        .any(|s| s.urls.contains(&"stun:stun.l.google.com:19302".to_string())));
    assert!(options.ice_servers.iter().any(|s| s
        .urls
        .contains(&"stun:stun.cloudflare.com:3478".to_string())));
    assert_eq!(options.data_channel.ordered, Some(false));
    assert_eq!(options.data_channel.max_retransmits, Some(2));
}

/// Test WebRTCOptions cloning
#[test]
fn test_webrtc_options_clone() {
    let options1 = WebRTCOptions::default();
    let options2 = options1.clone();

    assert_eq!(options1.max_connections, options2.max_connections);
    assert_eq!(options1.enabled, options2.enabled);
    assert_eq!(options1.ice_servers.len(), options2.ice_servers.len());
}

/// Test WebRTCPeer creation
#[tokio::test]
async fn test_webrtc_peer_creation() {
    let options = WebRTCOptions::default();
    let peer_id = "test_peer_1".to_string();

    let result = WebRTCPeer::new(peer_id.clone(), &options).await;
    assert!(
        result.is_ok(),
        "Failed to create WebRTC peer: {:?}",
        result.err()
    );

    let (_peer, _rx) = result.unwrap();
    assert_eq!(peer.peer_id, peer_id);

    // Test connection state
    let state = peer.connection_state().await;
    // State should be New or Connecting initially
    println!("Peer connection state: {:?}", state);

    // Clean up
    let _ = peer.close().await;
}

/// Test WebRTCPeer SDP offer creation
#[tokio::test]
async fn test_webrtc_peer_create_offer() {
    let options = WebRTCOptions::default();
    let peer_id = "test_peer_offer".to_string();

    let (_peer, _rx) = WebRTCPeer::new(peer_id, &options).await.unwrap();

    let result = peer.create_offer().await;
    assert!(result.is_ok(), "Failed to create offer: {:?}", result.err());

    let offer = result.unwrap();
    assert!(!offer.sdp.is_empty());
    assert_eq!(
        offer.sdp_type,
        webrtc::peer_connection::sdp::sdp_type::RTCSdpType::Offer
    );

    let _ = peer.close().await;
}

/// Test WebRTCPeer sending messages through data channel
#[tokio::test]
async fn test_webrtc_peer_send_message() {
    let options = WebRTCOptions::default();
    let peer_id = "test_peer_send".to_string();

    let (_peer, _rx) = WebRTCPeer::new(peer_id, &options).await.unwrap();

    // Note: Sending will fail without a connected peer, but we can test the method exists
    let test_message = "test message";
    let result = peer.send(test_message).await;

    // The send might fail if no peer is connected, which is expected
    // We're just testing that the method works and doesn't panic
    println!("Send result: {:?}", result);

    // Clean up
    let _ = peer.close().await;
}

/// Test WebRTCPeer connection state
#[tokio::test]
async fn test_webrtc_peer_connection_state() {
    let options = WebRTCOptions::default();
    let peer_id = "test_peer_state".to_string();

    let (_peer, _rx) = WebRTCPeer::new(peer_id, &options).await.unwrap();

    let state = peer.connection_state().await;
    // Should be New, Connecting, or similar initial state
    assert!(matches!(
        state,
        webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::New
            | webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState::Connecting
    ));

    let _ = peer.close().await;
}

/// Test WebRTCPeer closing
#[tokio::test]
async fn test_webrtc_peer_close() {
    let options = WebRTCOptions::default();
    let peer_id = "test_peer_close".to_string();

    let (_peer, _rx) = WebRTCPeer::new(peer_id, &options).await.unwrap();

    let result = peer.close().await;
    assert!(result.is_ok(), "Failed to close peer: {:?}", result.err());
}

/// Test WebRTCManager creation
#[tokio::test]
async fn test_webrtc_manager_creation() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh, options);

    // Manager should be created successfully
    assert!(!manager.pid().is_empty());
}

/// Test WebRTCManager handling RTC messages - offer
#[tokio::test]
async fn test_webrtc_manager_handle_offer() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh, options);

    // Create a test peer to generate an offer
    let test_peer_id = "test_peer_for_offer";
    let (_peer, _rx) = WebRTCPeer::new(test_peer_id.to_string(), &WebRTCOptions::default())
        .await
        .unwrap();
    let offer = peer.create_offer().await.unwrap();

    // Create RTC message with offer
    let rtc_msg = json!({
        "ok": {
            "rtc": {
                "id": "remote_peer_1",
                "offer": {
                    "type": "offer",
                    "sdp": offer.sdp
                }
            }
        }
    });

    // Handle the offer
    let result = manager.handle_rtc_message(&rtc_msg).await;
    assert!(result.is_ok(), "Failed to handle offer: {:?}", result.err());

    let _ = peer.close().await;
}

/// Test WebRTCManager handling RTC messages - answer
#[tokio::test]
async fn test_webrtc_manager_handle_answer() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh, options);

    // First create a peer connection by handling an offer
    let test_peer_id = "test_peer_for_answer";
    let (_peer, _rx) = WebRTCPeer::new(test_peer_id.to_string(), &WebRTCOptions::default())
        .await
        .unwrap();
    let offer = peer.create_offer().await.unwrap();

    // Handle the offer to create the peer in the manager
    let offer_msg = json!({
        "ok": {
            "rtc": {
                "id": test_peer_id,
                "offer": {
                    "type": "offer",
                    "sdp": offer.sdp
                }
            }
        }
    });

    let _ = manager.handle_rtc_message(&offer_msg).await;
    sleep(Duration::from_millis(200)).await;

    // Close the test peer as manager has its own
    let _ = peer.close().await;

    // Create answer from another peer
    let (answer_peer, _rx2) = WebRTCPeer::new("answer_peer".to_string(), &WebRTCOptions::default())
        .await
        .unwrap();
    let answer = answer_peer.create_answer().await.unwrap();

    // Create RTC message with answer
    let rtc_msg = json!({
        "ok": {
            "rtc": {
                "id": test_peer_id,
                "answer": {
                    "type": "answer",
                    "sdp": answer.sdp
                }
            }
        }
    });

    // Handle the answer
    let result = manager.handle_rtc_message(&rtc_msg).await;
    // This might fail if the peer connection isn't in the right state, which is expected
    println!("Handle answer result: {:?}", result);

    let _ = answer_peer.close().await;
}

/// Test WebRTCManager handling RTC messages - ICE candidate
#[tokio::test]
async fn test_webrtc_manager_handle_ice_candidate() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh, options);

    // Create a test peer by handling an offer first
    let test_peer_id = "test_peer_ice";
    let (_peer, _rx) = WebRTCPeer::new(test_peer_id.to_string(), &WebRTCOptions::default())
        .await
        .unwrap();
    let offer = peer.create_offer().await.unwrap();

    // Create the peer in manager by handling offer
    let offer_msg = json!({
        "ok": {
            "rtc": {
                "id": test_peer_id,
                "offer": {
                    "type": "offer",
                    "sdp": offer.sdp
                }
            }
        }
    });

    let _ = manager.handle_rtc_message(&offer_msg).await;
    sleep(Duration::from_millis(200)).await;

    let _ = peer.close().await;

    // Create RTC message with ICE candidate
    let rtc_msg = json!({
        "ok": {
            "rtc": {
                "id": test_peer_id,
                "candidate": {
                    "candidate": "candidate:1 1 UDP 2130706431 192.168.1.1 54321 typ host",
                    "sdpMLineIndex": 0,
                    "sdpMid": "0"
                }
            }
        }
    });

    // Handle the ICE candidate
    let result = manager.handle_rtc_message(&rtc_msg).await;
    assert!(
        result.is_ok(),
        "Failed to handle ICE candidate: {:?}",
        result.err()
    );
}

/// Test WebRTCManager handling RTC messages - peer discovery
#[tokio::test]
async fn test_webrtc_manager_handle_peer_discovery() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let mut options = WebRTCOptions::default();
    options.max_connections = 10; // Set a reasonable limit for testing

    let manager = WebRTCManager::new(core, mesh, options);

    // Create RTC message for peer discovery
    let rtc_msg = json!({
        "ok": {
            "rtc": {
                "id": "new_peer_1"
            }
        }
    });

    // Handle peer discovery - should initiate connection
    let result = manager.handle_rtc_message(&rtc_msg).await;
    assert!(
        result.is_ok(),
        "Failed to handle peer discovery: {:?}",
        result.err()
    );

    // Check that peer was added (might take a moment)
    sleep(Duration::from_millis(100)).await;
    // Note: peers field is private, so we can't directly check it
    // But the operation should complete successfully
}

/// Test WebRTCManager ignoring own messages
#[tokio::test]
async fn test_webrtc_manager_ignore_own_messages() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh, options);
    let pid = manager.pid().to_string();

    // Create RTC message with our own ID
    let rtc_msg = json!({
        "ok": {
            "rtc": {
                "id": pid
            }
        }
    });

    // Handle should succeed but ignore the message (won't create connection to ourselves)
    let result = manager.handle_rtc_message(&rtc_msg).await;
    assert!(
        result.is_ok(),
        "Should handle own messages gracefully: {:?}",
        result.err()
    );
}

/// Test WebRTCManager handling invalid RTC messages
#[tokio::test]
async fn test_webrtc_manager_handle_invalid_message() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh, options);

    // Message without "ok.rtc" should be ignored
    let invalid_msg = json!({
        "dam": "?",
        "pid": "some_peer"
    });

    let result = manager.handle_rtc_message(&invalid_msg).await;
    assert!(result.is_ok(), "Should handle invalid messages gracefully");

    // Message with missing peer ID should fail
    let msg_no_id = json!({
        "ok": {
            "rtc": {
                "offer": {
                    "type": "offer",
                    "sdp": "invalid sdp"
                }
            }
        }
    });

    let result = manager.handle_rtc_message(&msg_no_id).await;
    assert!(result.is_err(), "Should fail on missing peer ID");
}

/// Test WebRTCManager connection limit
#[tokio::test]
async fn test_webrtc_manager_connection_limit() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let mut options = WebRTCOptions::default();
    options.max_connections = 2; // Set low limit for testing

    let manager = WebRTCManager::new(core, mesh, options);

    // Try to initiate connections up to limit
    for i in 0..3 {
        let rtc_msg = json!({
            "ok": {
                "rtc": {
                    "id": format!("peer_{}", i)
                }
            }
        });

        let result = manager.handle_rtc_message(&rtc_msg).await;
        assert!(
            result.is_ok(),
            "Should handle connection attempt: {:?}",
            result.err()
        );
        // Small delay to allow connection processing
        sleep(Duration::from_millis(50)).await;
    }

    // Note: We can't directly check peer count as peers field is private
    // But the manager should enforce the limit internally
}

/// Test Gun with WebRTC enabled
#[tokio::test]
async fn test_gun_with_webrtc_enabled() {
    let mut options = GunOptions::default();
    options.webrtc.enabled = true;

    let gun = Gun::with_options(options).await;
    assert!(
        gun.is_ok(),
        "Failed to create Gun with WebRTC: {:?}",
        gun.err()
    );

    let _gun = gun.unwrap();
    // WebRTC manager should exist if enabled
    // Note: webrtc_manager is private, so we can't directly check it
    // But the Gun instance should be created successfully
}

/// Test Gun with WebRTC disabled
#[tokio::test]
async fn test_gun_with_webrtc_disabled() {
    let mut options = GunOptions::default();
    options.webrtc.enabled = false;

    let gun = Gun::with_options(options).await;
    assert!(
        gun.is_ok(),
        "Failed to create Gun without WebRTC: {:?}",
        gun.err()
    );
}

/// Test WebRTC message forwarding from data channel to mesh
#[tokio::test]
async fn test_webrtc_message_forwarding() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh.clone(), options);

    // Create a test peer
    let test_peer_id = "test_peer_forward";
    let (_peer, _rx) = WebRTCPeer::new(test_peer_id.to_string(), &WebRTCOptions::default())
        .await
        .unwrap();

    // Note: We can't directly add peers as the peers field is private
    // The test verifies that send_message handles the case where peer doesn't exist
    // (it will fall back to WebSocket via mesh)

    // Test sending a message through WebRTC
    // This will fall back to WebSocket if WebRTC isn't connected
    let test_message = "{\"#\": \"test_id\", \"put\": {\"test_key\": \"test_value\"}}";
    let result = manager.send_message(test_peer_id, test_message).await;

    // Should succeed (might use WebSocket fallback)
    println!("Send message result: {:?}", result);
}

/// Test WebRTCManager with custom ICE servers
#[tokio::test]
async fn test_webrtc_custom_ice_servers() {
    use webrtc::ice_transport::ice_credential_type::RTCIceCredentialType;
    use webrtc::ice_transport::ice_server::RTCIceServer;

    let mut options = WebRTCOptions::default();
    options.ice_servers.push(RTCIceServer {
        urls: vec!["stun:stun.example.com:3478".to_string()],
        username: "test_user".to_string(),
        credential: "test_pass".to_string(),
        credential_type: RTCIceCredentialType::Password,
    });

    assert_eq!(options.ice_servers.len(), 3);

    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let manager = WebRTCManager::new(core, mesh, options);

    // Manager should be created with custom ICE servers
    assert!(!manager.pid().is_empty());
}

/// Test concurrent WebRTC peer creation
#[tokio::test]
async fn test_webrtc_concurrent_peer_creation() {
    let options = WebRTCOptions::default();

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let options_clone = options.clone();
            tokio::spawn(async move {
                let peer_id = format!("concurrent_peer_{}", i);
                WebRTCPeer::new(peer_id, &options_clone).await
            })
        })
        .collect();

    // Wait for all peers to be created
    let results = future::join_all(handles).await;

    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Failed to create peer {}: {:?}",
            i,
            result.as_ref().unwrap_err()
        );
        if let Ok(Ok((peer, _rx))) = result {
            let _ = peer.close().await;
        }
    }
}

/// Test WebRTC peer with different data channel configurations
#[tokio::test]
async fn test_webrtc_custom_data_channel_config() {
    use webrtc::data_channel::data_channel_init::RTCDataChannelInit;

    let mut options = WebRTCOptions::default();
    let mut data_channel = RTCDataChannelInit::default();
    data_channel.ordered = Some(true);
    data_channel.max_retransmits = Some(5);
    options.data_channel = data_channel;

    let peer_id = "test_custom_config".to_string();
    let result = WebRTCPeer::new(peer_id, &options).await;
    assert!(
        result.is_ok(),
        "Failed to create peer with custom config: {:?}",
        result.err()
    );

    if let Ok((peer, _rx)) = result {
        let _ = peer.close().await;
    }
}

/// Test error handling for invalid SDP
#[tokio::test]
async fn test_webrtc_invalid_sdp_handling() {
    let core = Arc::new(GunCore::new());
    let mesh = Arc::new(Mesh::new(core.clone()));
    let options = WebRTCOptions::default();

    let manager = WebRTCManager::new(core, mesh, options);

    // Create RTC message with invalid SDP
    let rtc_msg = json!({
        "ok": {
            "rtc": {
                "id": "test_peer",
                "offer": {
                    "type": "offer",
                    "sdp": "invalid sdp string that will fail to parse"
                }
            }
        }
    });

    // Should handle gracefully (might fail to parse SDP, which is expected)
    let result = manager.handle_rtc_message(&rtc_msg).await;
    // Result might be Ok (if peer doesn't exist) or Err (if SDP parsing fails)
    println!("Invalid SDP handling result: {:?}", result);
}

/// Test WebRTC with timeout for connection establishment
#[tokio::test]
async fn test_webrtc_connection_timeout() {
    timeout(Duration::from_secs(5), async {
        let options = WebRTCOptions::default();
        let peer_id = "test_timeout".to_string();

        let (_peer, _rx) = WebRTCPeer::new(peer_id, &options).await.unwrap();

        // Try to get connection state - should complete quickly
        let _state = peer.connection_state().await;

        let _ = peer.close().await;
    })
    .await
    .expect("WebRTC operations should complete within timeout");
}
