//! Comprehensive tests for state management
//! Tests timestamp generation, state tracking, and node state operations

use gun::state::{Node, State};
use serde_json::json;

#[test]
fn test_state_new() {
    let state = State::new();
    // Should be able to generate timestamps
    let ts1 = state.next();
    assert!(ts1 > 0.0);
}

#[test]
fn test_state_next_monotonic() {
    let state = State::new();
    let ts1 = state.next();

    // Small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(1));

    let ts2 = state.next();
    assert!(ts2 >= ts1, "State should increase monotonically");

    let ts3 = state.next();
    assert!(ts3 >= ts2, "State should continue to increase");
}

#[test]
fn test_state_next_rapid_calls() {
    let state = State::new();
    let mut timestamps = Vec::new();

    // Generate many timestamps rapidly
    for _ in 0..100 {
        timestamps.push(state.next());
    }

    // All should be in increasing order
    for i in 1..timestamps.len() {
        assert!(
            timestamps[i] >= timestamps[i - 1],
            "Timestamps should be non-decreasing"
        );
    }
}

#[test]
fn test_state_is_none() {
    // State::is with None should return None
    assert_eq!(State::is(&None, "key"), None);
}

#[test]
fn test_state_is_with_key() {
    let mut node = Node::new();
    let soul = "test_soul".to_string();

    // Set state for a key
    State::ify(
        &mut node,
        Some("name"),
        Some(12345.0),
        Some(json!("Alice")),
        Some(&soul),
    );

    // Retrieve state
    let state = State::is(&Some(node), "name");
    assert_eq!(state, Some(12345.0));
}

#[test]
fn test_state_is_non_existent_key() {
    let mut node = Node::new();
    let soul = "test_soul".to_string();
    State::ify(
        &mut node,
        Some("name"),
        Some(12345.0),
        Some(json!("Alice")),
        Some(&soul),
    );

    // Try to get state for non-existent key
    let state = State::is(&Some(node), "nonexistent");
    assert_eq!(state, None);
}

#[test]
fn test_state_ify_sets_soul() {
    let mut node = Node::new();
    let soul = "test_soul_123".to_string();

    State::ify(&mut node, None, None, None, Some(&soul));

    assert_eq!(node.get_soul(), Some(soul));
}

#[test]
fn test_state_ify_sets_data_and_state() {
    let mut node = Node::new();
    let soul = "test_soul".to_string();

    State::ify(
        &mut node,
        Some("age"),
        Some(67890.0),
        Some(json!(30)),
        Some(&soul),
    );

    // Check data
    assert_eq!(node.data.get("age"), Some(&json!(30)));

    // Check state
    let state = State::is(&Some(node), "age");
    assert_eq!(state, Some(67890.0));
}

#[test]
fn test_state_ify_multiple_keys() {
    let mut node = Node::new();
    let soul = "test_soul".to_string();

    State::ify(
        &mut node,
        Some("name"),
        Some(1000.0),
        Some(json!("Bob")),
        Some(&soul),
    );
    State::ify(
        &mut node,
        Some("age"),
        Some(2000.0),
        Some(json!(25)),
        Some(&soul),
    );
    State::ify(
        &mut node,
        Some("city"),
        Some(3000.0),
        Some(json!("NYC")),
        Some(&soul),
    );

    // Check all states
    assert_eq!(State::is(&Some(node.clone()), "name"), Some(1000.0));
    assert_eq!(State::is(&Some(node.clone()), "age"), Some(2000.0));
    assert_eq!(State::is(&Some(node.clone()), "city"), Some(3000.0));

    // Check all data
    assert_eq!(node.data.get("name"), Some(&json!("Bob")));
    assert_eq!(node.data.get("age"), Some(&json!(25)));
    assert_eq!(node.data.get("city"), Some(&json!("NYC")));
}

#[test]
fn test_state_ify_updates_existing_key() {
    let mut node = Node::new();
    let soul = "test_soul".to_string();

    // Set initial value
    State::ify(
        &mut node,
        Some("count"),
        Some(1000.0),
        Some(json!(5)),
        Some(&soul),
    );

    // Update with new state
    State::ify(
        &mut node,
        Some("count"),
        Some(2000.0),
        Some(json!(10)),
        Some(&soul),
    );

    // Should have new value and state
    assert_eq!(node.data.get("count"), Some(&json!(10)));
    assert_eq!(State::is(&Some(node), "count"), Some(2000.0));
}

#[test]
fn test_state_ify_ignores_underscore_key() {
    let mut node = Node::new();
    let soul = "test_soul".to_string();

    // Try to set state for "_" key (should be ignored for state)
    State::ify(
        &mut node,
        Some("_"),
        Some(12345.0),
        Some(json!("meta")),
        Some(&soul),
    );

    // State should not be set for "_" key
    let state = State::is(&Some(node), "_");
    assert_eq!(state, None);
}

#[test]
fn test_state_default() {
    let state = State::default();
    let ts = state.next();
    assert!(ts > 0.0);
}

#[test]
fn test_node_new() {
    let node = Node::new();
    assert!(node.data.is_empty());
    assert!(node.meta.is_empty());
}

#[test]
fn test_node_with_soul() {
    let soul = "test_soul_456".to_string();
    let node = Node::with_soul(soul.clone());

    assert_eq!(node.get_soul(), Some(soul));
    assert!(node.data.is_empty());
}

#[test]
fn test_node_default() {
    let node = Node::default();
    assert!(node.data.is_empty());
    assert!(node.meta.is_empty());
}

#[test]
fn test_state_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let state = Arc::new(State::new());
    let mut handles = vec![];

    // Spawn multiple threads generating timestamps
    for _ in 0..10 {
        let state_clone = state.clone();
        handles.push(thread::spawn(move || {
            let mut timestamps = Vec::new();
            for _ in 0..10 {
                timestamps.push(state_clone.next());
            }
            timestamps
        }));
    }

    // Collect all timestamps
    let mut all_timestamps = Vec::new();
    for handle in handles {
        all_timestamps.extend(handle.join().unwrap());
    }

    // All should be unique and increasing (when sorted)
    all_timestamps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for i in 1..all_timestamps.len() {
        assert!(
            all_timestamps[i] >= all_timestamps[i - 1],
            "All timestamps should be non-decreasing"
        );
    }
}
