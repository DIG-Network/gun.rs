//! Comprehensive tests for graph operations
//! Tests node storage, retrieval, and HAM merge logic

use gun::graph::Graph;
use gun::state::{Node, State};
use serde_json::json;

#[test]
fn test_graph_new() {
    let graph = Graph::new();
    assert!(!graph.has("test"));
}

#[test]
fn test_graph_put_get() {
    let graph = Graph::new();
    let node = Node::with_soul("soul1".to_string());

    graph.put("soul1", node.clone()).unwrap();

    let retrieved = graph.get("soul1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().get_soul(), Some("soul1".to_string()));
}

#[test]
fn test_graph_get_non_existent() {
    let graph = Graph::new();
    let retrieved = graph.get("nonexistent");
    assert!(retrieved.is_none());
}

#[test]
fn test_graph_has() {
    let graph = Graph::new();
    let node = Node::with_soul("soul1".to_string());

    assert!(!graph.has("soul1"));
    graph.put("soul1", node).unwrap();
    assert!(graph.has("soul1"));
}

#[test]
fn test_graph_update() {
    let graph = Graph::new();
    let mut node1 = Node::with_soul("soul1".to_string());
    node1.data.insert("name".to_string(), json!("Alice"));

    graph.put("soul1", node1).unwrap();

    let mut node2 = Node::with_soul("soul1".to_string());
    node2.data.insert("name".to_string(), json!("Bob"));

    graph.put("soul1", node2).unwrap();

    let retrieved = graph.get("soul1").unwrap();
    assert_eq!(retrieved.data.get("name"), Some(&json!("Bob")));
}

#[test]
fn test_graph_merge_same_timestamp() {
    let graph = Graph::new();
    let state = State::new();

    let mut existing = Node::with_soul("soul1".to_string());
    State::ify(
        &mut existing,
        Some("name"),
        Some(1000.0),
        Some(json!("Alice")),
        Some("soul1"),
    );

    let mut incoming = Node::with_soul("soul1".to_string());
    State::ify(
        &mut incoming,
        Some("name"),
        Some(1000.0),
        Some(json!("Bob")),
        Some("soul1"),
    );

    graph.put("soul1", existing.clone()).unwrap();

    // Merge - same timestamp, incoming wins (>= comparison)
    let merged = graph.merge("soul1", &incoming, || state.next()).unwrap();

    // Should have incoming value (due to >= comparison)
    assert_eq!(merged.data.get("name"), Some(&json!("Bob")));
}

#[test]
fn test_graph_merge_different_timestamps() {
    let graph = Graph::new();
    let state = State::new();

    let mut existing = Node::with_soul("soul1".to_string());
    State::ify(
        &mut existing,
        Some("name"),
        Some(1000.0),
        Some(json!("Alice")),
        Some("soul1"),
    );

    let mut incoming = Node::with_soul("soul1".to_string());
    State::ify(
        &mut incoming,
        Some("name"),
        Some(2000.0),
        Some(json!("Bob")),
        Some("soul1"),
    );

    graph.put("soul1", existing.clone()).unwrap();

    // Merge - newer timestamp wins
    let merged = graph.merge("soul1", &incoming, || state.next()).unwrap();

    assert_eq!(merged.data.get("name"), Some(&json!("Bob")));
}

#[test]
fn test_graph_merge_older_timestamp() {
    let graph = Graph::new();
    let state = State::new();

    let mut existing = Node::with_soul("soul1".to_string());
    State::ify(
        &mut existing,
        Some("name"),
        Some(2000.0),
        Some(json!("Alice")),
        Some("soul1"),
    );

    let mut incoming = Node::with_soul("soul1".to_string());
    State::ify(
        &mut incoming,
        Some("name"),
        Some(1000.0),
        Some(json!("Bob")),
        Some("soul1"),
    );

    graph.put("soul1", existing.clone()).unwrap();

    // Merge - older timestamp should not overwrite
    let merged = graph.merge("soul1", &incoming, || state.next()).unwrap();

    // Should keep existing value (newer timestamp)
    assert_eq!(merged.data.get("name"), Some(&json!("Alice")));
}

#[test]
fn test_graph_merge_multiple_keys() {
    let graph = Graph::new();
    let state = State::new();

    let mut existing = Node::with_soul("soul1".to_string());
    State::ify(
        &mut existing,
        Some("name"),
        Some(1000.0),
        Some(json!("Alice")),
        Some("soul1"),
    );
    State::ify(
        &mut existing,
        Some("age"),
        Some(1000.0),
        Some(json!(30)),
        Some("soul1"),
    );

    let mut incoming = Node::with_soul("soul1".to_string());
    State::ify(
        &mut incoming,
        Some("name"),
        Some(2000.0),
        Some(json!("Bob")),
        Some("soul1"),
    );
    State::ify(
        &mut incoming,
        Some("city"),
        Some(2000.0),
        Some(json!("NYC")),
        Some("soul1"),
    );

    graph.put("soul1", existing.clone()).unwrap();

    let merged = graph.merge("soul1", &incoming, || state.next()).unwrap();

    // Newer name should be updated
    assert_eq!(merged.data.get("name"), Some(&json!("Bob")));
    // Older age should remain
    assert_eq!(merged.data.get("age"), Some(&json!(30)));
    // New key should be added
    assert_eq!(merged.data.get("city"), Some(&json!("NYC")));
}

#[test]
fn test_graph_merge_new_node() {
    let graph = Graph::new();
    let state = State::new();

    let incoming = Node::with_soul("soul1".to_string());

    // Merge when node doesn't exist
    let merged = graph.merge("soul1", &incoming, || state.next()).unwrap();

    assert_eq!(merged.get_soul(), Some("soul1".to_string()));
    assert!(graph.has("soul1"));
}

#[test]
fn test_graph_all_nodes() {
    let graph = Graph::new();

    for i in 0..10 {
        let soul = format!("soul{}", i);
        let node = Node::with_soul(soul.clone());
        graph.put(&soul, node).unwrap();
    }

    let all = graph.all_nodes();
    assert_eq!(all.len(), 10);
}
