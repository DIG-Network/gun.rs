//! Comprehensive tests for SEA module
//! Tests pair, sign, verify, encrypt, decrypt, secret, user functions

use gun::sea::secret;
use gun::sea::{create_user, decrypt, encrypt, pair, sign, verify};
use serde_json::json;

#[tokio::test]
async fn test_sea_pair_generation() {
    let keypair = pair().await.unwrap();

    // Should have pub and priv keys
    assert!(!keypair.pub_key.is_empty());
    assert!(!keypair.priv_key.is_empty());

    // Pub key should be in x.y format
    assert!(keypair.pub_key.contains('.'));
    let parts: Vec<&str> = keypair.pub_key.split('.').collect();
    assert_eq!(parts.len(), 2);
}

#[tokio::test]
async fn test_sea_pair_epub_keys() {
    let keypair = pair().await.unwrap();

    // Should have encryption keys
    assert!(keypair.epub_key.is_some());
    assert!(keypair.epriv_key.is_some());

    let epub = keypair.epub_key.unwrap();
    assert!(epub.contains('.'));
}

#[tokio::test]
async fn test_sea_pair_unique() {
    let pair1 = pair().await.unwrap();
    let pair2 = pair().await.unwrap();

    // Should generate different key pairs
    assert_ne!(pair1.pub_key, pair2.pub_key);
    assert_ne!(pair1.priv_key, pair2.priv_key);
}

#[tokio::test]
async fn test_sea_sign_data() {
    let keypair = pair().await.unwrap();
    let data = json!({"message": "hello world"});

    let signed = sign(&data, &keypair).await.unwrap();

    // Should have "m" (message) and "s" (signature) fields
    assert!(signed.get("m").is_some());
    assert!(signed.get("s").is_some());

    // Message should match original data (serialized)
    let message_str = signed.get("m").unwrap().as_str().unwrap();
    let original_str = serde_json::to_string(&data).unwrap();
    assert_eq!(message_str, original_str);
}

#[tokio::test]
async fn test_sea_sign_verify() {
    let keypair = pair().await.unwrap();
    let data = json!({"test": "data"});

    // Sign data
    let signed = sign(&data, &keypair).await.unwrap();

    // Verify signature (returns verified message data)
    let verified = verify(&signed, &keypair.pub_key).await;
    assert!(verified.is_ok());
    let verified_data = verified.unwrap();
    // Should return the original data (parsed from message)
    assert_eq!(verified_data, data);
}

#[tokio::test]
async fn test_sea_verify_wrong_key() {
    let keypair1 = pair().await.unwrap();
    let keypair2 = pair().await.unwrap();
    let data = json!({"test": "data"});

    // Sign with keypair1
    let signed = sign(&data, &keypair1).await.unwrap();

    // Try to verify with keypair2's public key
    let verified = verify(&signed, &keypair2.pub_key).await;
    // Should fail verification
    assert!(verified.is_err());
}

#[tokio::test]
async fn test_sea_verify_tampered_data() {
    let keypair = pair().await.unwrap();
    let data = json!({"test": "data"});

    let mut signed = sign(&data, &keypair).await.unwrap();

    // Tamper with the message
    if let Some(m) = signed.get_mut("m") {
        *m = json!("tampered");
    }

    // Verification should fail (tampered data)
    let verified = verify(&signed, &keypair.pub_key).await;
    assert!(verified.is_err());
}

#[tokio::test]
async fn test_sea_encrypt_decrypt() {
    let keypair = pair().await.unwrap();
    let data = json!({"secret": "message"});

    // Encrypt (self-encryption, no their_epub)
    let encrypted = encrypt(&data, &keypair, None).await.unwrap();

    // Should have ct, iv, s fields
    assert!(encrypted.get("ct").is_some());
    assert!(encrypted.get("iv").is_some());
    assert!(encrypted.get("s").is_some());

    // Decrypt
    let decrypted = decrypt(&encrypted, &keypair, None).await.unwrap();

    // Should match original
    assert_eq!(decrypted, data);
}

#[tokio::test]
async fn test_sea_encrypt_decrypt_with_epub() {
    let alice = pair().await.unwrap();
    let bob = pair().await.unwrap();

    let data = json!({"message": "hello bob"});

    // Alice encrypts for Bob
    let bob_epub = bob.epub_key.as_ref().unwrap();
    let encrypted = encrypt(&data, &alice, Some(bob_epub)).await.unwrap();

    // Bob decrypts
    let alice_epub = alice.epub_key.as_ref().unwrap();
    let decrypted = decrypt(&encrypted, &bob, Some(alice_epub)).await.unwrap();

    assert_eq!(decrypted, data);
}

#[tokio::test]
async fn test_sea_encrypt_wrong_key() {
    let alice = pair().await.unwrap();
    let bob = pair().await.unwrap();
    let charlie = pair().await.unwrap();

    let data = json!({"secret": "data"});

    // Alice encrypts for Bob
    let bob_epub = bob.epub_key.as_ref().unwrap();
    let encrypted = encrypt(&data, &alice, Some(bob_epub)).await.unwrap();

    // Charlie tries to decrypt (should fail)
    let alice_epub = alice.epub_key.as_ref().unwrap();
    let result = decrypt(&encrypted, &charlie, Some(alice_epub)).await;

    // Should fail (wrong key)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sea_secret_derivation() {
    let alice = pair().await.unwrap();
    let bob = pair().await.unwrap();

    let alice_epub = alice.epub_key.as_ref().unwrap();
    let alice_epriv = alice.epriv_key.as_ref().unwrap();
    let bob_epub = bob.epub_key.as_ref().unwrap();
    let bob_epriv = bob.epriv_key.as_ref().unwrap();

    // Alice derives secret from Bob's pub key
    let secret1 = secret(bob_epub, alice_epriv, alice_epub).await.unwrap();

    // Bob derives secret from Alice's pub key
    let secret2 = secret(alice_epub, bob_epriv, bob_epub).await.unwrap();

    // Should be the same (ECDH property)
    assert_eq!(secret1, secret2);
}

#[tokio::test]
async fn test_sea_secret_different_peers() {
    let alice = pair().await.unwrap();
    let bob = pair().await.unwrap();
    let charlie = pair().await.unwrap();

    let alice_epub = alice.epub_key.as_ref().unwrap();
    let alice_epriv = alice.epriv_key.as_ref().unwrap();
    let bob_epub = bob.epub_key.as_ref().unwrap();
    let charlie_epub = charlie.epub_key.as_ref().unwrap();

    let secret_alice_bob = secret(bob_epub, alice_epriv, alice_epub).await.unwrap();
    let secret_alice_charlie = secret(charlie_epub, alice_epriv, alice_epub).await.unwrap();

    // Should be different
    assert_ne!(secret_alice_bob, secret_alice_charlie);
}

#[tokio::test]
async fn test_sea_user_create() {
    use gun::Gun;
    use std::sync::Arc;
    
    let gun = Gun::new();
    let chain = gun.root();
    let user = create_user(chain, Some("alice".to_string()), "password123").await.unwrap();

    assert!(!user.pair.pub_key.is_empty());
    assert_eq!(user.alias, Some("alice".to_string()));
}

#[tokio::test]
async fn test_sea_user_create_no_alias() {
    use gun::Gun;
    use std::sync::Arc;
    
    let gun = Gun::new();
    let chain = gun.root();
    let user = create_user(chain, None, "password123").await.unwrap();

    assert!(!user.pair.pub_key.is_empty());
    assert_eq!(user.alias, None);
}

#[tokio::test]
async fn test_sea_encrypt_various_types() {
    let keypair = pair().await.unwrap();

    // Test encrypting different data types
    let string_data = json!("hello");
    let number_data = json!(42);
    let bool_data = json!(true);
    let object_data = json!({"key": "value"});
    let null_data = json!(null);

    // All should encrypt/decrypt successfully
    let enc_str = encrypt(&string_data, &keypair, None).await.unwrap();
    let dec_str = decrypt(&enc_str, &keypair, None).await.unwrap();
    assert_eq!(dec_str, string_data);

    let enc_num = encrypt(&number_data, &keypair, None).await.unwrap();
    let dec_num = decrypt(&enc_num, &keypair, None).await.unwrap();
    assert_eq!(dec_num, number_data);

    let enc_bool = encrypt(&bool_data, &keypair, None).await.unwrap();
    let dec_bool = decrypt(&enc_bool, &keypair, None).await.unwrap();
    assert_eq!(dec_bool, bool_data);

    let enc_obj = encrypt(&object_data, &keypair, None).await.unwrap();
    let dec_obj = decrypt(&enc_obj, &keypair, None).await.unwrap();
    assert_eq!(dec_obj, object_data);

    let enc_null = encrypt(&null_data, &keypair, None).await.unwrap();
    let dec_null = decrypt(&enc_null, &keypair, None).await.unwrap();
    assert_eq!(dec_null, null_data);
}
