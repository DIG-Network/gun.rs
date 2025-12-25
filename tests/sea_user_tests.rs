//! Comprehensive tests for SEA user authentication and recall
//! Tests user creation, authentication, and session recall with graph storage

use gun::Gun;
use gun::sea::{authenticate, create_user, recall};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_user_create_with_graph() {
    let gun = Gun::new();
    let chain = gun.root();
    
    let user = create_user(chain.clone(), Some("testuser".to_string()), "password123")
        .await
        .unwrap();
    
    assert!(!user.pair.pub_key.is_empty());
    assert_eq!(user.alias, Some("testuser".to_string()));
    
    // Verify user data is stored in graph
    let user_soul = format!("~{}", user.pair.pub_key);
    let mut found = false;
    chain.get(&user_soul).once(|data, _key| {
        if data.is_object() {
            found = true;
            assert!(data.get("pub").is_some());
            assert!(data.get("hash").is_some());
            assert!(data.get("salt").is_some());
        }
    }).await.unwrap();
    
    assert!(found, "User data should be stored in graph");
}

#[tokio::test]
async fn test_user_authenticate_success() {
    let gun = Gun::new();
    let chain = gun.root();
    
    // Create user
    let created_user = create_user(chain.clone(), Some("authuser".to_string()), "mypassword")
        .await
        .unwrap();
    
    // Authenticate with correct password
    let authenticated = authenticate(chain.clone(), "authuser", "mypassword")
        .await
        .unwrap();
    
    assert_eq!(authenticated.pair.pub_key, created_user.pair.pub_key);
    assert_eq!(authenticated.alias, Some("authuser".to_string()));
    assert!(!authenticated.pair.priv_key.is_empty());
}

#[tokio::test]
async fn test_user_authenticate_wrong_password() {
    let gun = Gun::new();
    let chain = gun.root();
    
    // Create user
    create_user(chain.clone(), Some("wrongpass".to_string()), "correctpass")
        .await
        .unwrap();
    
    // Try to authenticate with wrong password
    let result = authenticate(chain.clone(), "wrongpass", "wrongpass")
        .await;
    
    assert!(result.is_err(), "Authentication should fail with wrong password");
}

#[tokio::test]
async fn test_user_authenticate_nonexistent() {
    let gun = Gun::new();
    let chain = gun.root();
    
    // Try to authenticate non-existent user
    let result = authenticate(chain.clone(), "nonexistent", "password")
        .await;
    
    assert!(result.is_err(), "Authentication should fail for non-existent user");
}

#[tokio::test]
async fn test_user_recall_from_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    // Create temporary file with session data
    let mut temp_file = NamedTempFile::new().unwrap();
    let session_data = json!({
        "pub": "testpub",
        "priv": "testpriv",
        "epub": "testepub",
        "epriv": "testepriv",
        "alias": "recalluser",
        "exp": chrono::Utc::now().timestamp_millis() as f64
    });
    writeln!(temp_file, "{}", serde_json::to_string(&session_data).unwrap()).unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    let gun = Gun::new();
    let chain = gun.root();
    
    // Recall session
    let recalled = recall(Some(chain.clone()), Some(file_path))
        .await
        .unwrap();
    
    assert!(recalled.is_some());
    let user = recalled.unwrap();
    assert_eq!(user.alias, Some("recalluser".to_string()));
    assert_eq!(user.pair.pub_key, "testpub");
}

#[tokio::test]
async fn test_user_recall_expired() {
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    // Create temporary file with expired session data
    let mut temp_file = NamedTempFile::new().unwrap();
    let expired_time = chrono::Utc::now().timestamp_millis() as f64 - (13.0 * 60.0 * 60.0 * 1000.0); // 13 hours ago (expired)
    let session_data = json!({
        "pub": "testpub",
        "priv": "testpriv",
        "epub": "testepub",
        "epriv": "testepriv",
        "alias": "expireduser",
        "exp": expired_time
    });
    writeln!(temp_file, "{}", serde_json::to_string(&session_data).unwrap()).unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    
    let gun = Gun::new();
    let chain = gun.root();
    
    // Recall expired session
    let recalled = recall(Some(chain.clone()), Some(file_path))
        .await
        .unwrap();
    
    assert!(recalled.is_none(), "Expired session should return None");
}

#[tokio::test]
async fn test_user_recall_no_file() {
    let gun = Gun::new();
    let chain = gun.root();
    
    // Recall with no file path
    let recalled = recall(Some(chain.clone()), None)
        .await
        .unwrap();
    
    assert!(recalled.is_none(), "No file should return None");
}

#[tokio::test]
async fn test_user_create_and_authenticate_flow() {
    let gun = Gun::new();
    let chain = gun.root();
    
    // Create user
    let created = create_user(chain.clone(), Some("flowuser".to_string()), "flowpass")
        .await
        .unwrap();
    
    // Authenticate
    let authenticated = authenticate(chain.clone(), "flowuser", "flowpass")
        .await
        .unwrap();
    
    // Verify keys match
    assert_eq!(created.pair.pub_key, authenticated.pair.pub_key);
    assert_eq!(created.pair.epub_key, authenticated.pair.epub_key);
    
    // Verify we can use the authenticated keys for signing
    use gun::sea::{sign, verify};
    let data = json!({"test": "data"});
    let signed = sign(&data, &authenticated.pair).await.unwrap();
    let verified = verify(&signed, &authenticated.pair.pub_key).await.unwrap();
    assert_eq!(verified, data);
}

