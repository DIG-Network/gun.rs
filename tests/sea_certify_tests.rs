//! Tests for SEA.certify() - Certificate-Based Access Control

use gun::sea::{certify, pair, Certificants, CertifyOptions, Policy, RadixPolicy};
use std::collections::HashMap;

#[tokio::test]
async fn test_certify_wildcard() {
    let authority = pair().await.unwrap();
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("inbox".to_string()),
        &authority,
        CertifyOptions::default(),
    )
    .await
    .unwrap();

    assert!(cert.starts_with("SEA"));
}

#[tokio::test]
async fn test_certify_single_certificant() {
    let authority = pair().await.unwrap();
    let certificant = pair().await.unwrap();

    let cert = certify(
        Certificants::List(vec![certificant.pub_key.clone()]),
        Policy::String("inbox".to_string()),
        &authority,
        CertifyOptions::default(),
    )
    .await
    .unwrap();

    assert!(cert.starts_with("SEA"));
}

#[tokio::test]
async fn test_certify_with_expiry() {
    let authority = pair().await.unwrap();
    let expiry = chrono::Utc::now().timestamp_millis() as f64 + 3600000.0; // 1 hour from now

    let cert = certify(
        Certificants::Wildcard,
        Policy::String("test".to_string()),
        &authority,
        CertifyOptions {
            expiry: Some(expiry),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(cert.starts_with("SEA"));
}

#[tokio::test]
async fn test_certify_radix_policy() {
    let authority = pair().await.unwrap();
    let mut patterns = HashMap::new();
    patterns.insert("*".to_string(), "inbox".to_string());

    let cert = certify(
        Certificants::Wildcard,
        Policy::Radix(RadixPolicy { patterns }),
        &authority,
        CertifyOptions::default(),
    )
    .await
    .unwrap();

    assert!(cert.starts_with("SEA"));
}

#[tokio::test]
async fn test_certify_raw_format() {
    let authority = pair().await.unwrap();
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("test".to_string()),
        &authority,
        CertifyOptions {
            raw: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Raw format should not have "SEA" prefix
    assert!(!cert.starts_with("SEA"));
}

#[tokio::test]
async fn test_certify_verify_signature() {
    let authority = pair().await.unwrap();
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("test".to_string()),
        &authority,
        CertifyOptions::default(),
    )
    .await
    .unwrap();

    // Verify certificate with full signature verification
    use gun::sea::verify_certificate;
    let verified = verify_certificate(&cert, &authority.pub_key).await;
    
    assert!(verified.is_ok(), "Certificate should verify with correct authority key");
    let certificate = verified.unwrap();
    assert!(matches!(certificate.certificants, Certificants::Wildcard));
}

#[tokio::test]
async fn test_certify_verify_wrong_authority() {
    let authority1 = pair().await.unwrap();
    let authority2 = pair().await.unwrap();
    
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("test".to_string()),
        &authority1,
        CertifyOptions::default(),
    )
    .await
    .unwrap();

    // Try to verify with wrong authority key
    use gun::sea::verify_certificate;
    let verified = verify_certificate(&cert, &authority2.pub_key).await;
    
    // Should fail verification
    assert!(verified.is_err(), "Certificate should fail verification with wrong authority key");
}

#[tokio::test]
async fn test_certify_verify_with_expiry() {
    let authority = pair().await.unwrap();
    let expiry = chrono::Utc::now().timestamp_millis() as f64 + 3600000.0; // 1 hour from now
    
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("test".to_string()),
        &authority,
        CertifyOptions {
            expiry: Some(expiry),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    use gun::sea::verify_certificate;
    let verified = verify_certificate(&cert, &authority.pub_key).await.unwrap();
    
    assert!(verified.expiry.is_some());
    assert_eq!(verified.expiry.unwrap(), expiry);
}

#[tokio::test]
async fn test_certify_verify_permissions() {
    let authority = pair().await.unwrap();
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("inbox".to_string()),
        &authority,
        CertifyOptions::default(),
    )
    .await
    .unwrap();

    use gun::sea::{verify_certificate, check_permission};
    let certificate = verify_certificate(&cert, &authority.pub_key).await.unwrap();
    
    // Check permissions
    assert!(check_permission(&certificate, "inbox", "write"));
    assert!(check_permission(&certificate, "inbox/messages", "write"));
    assert!(!check_permission(&certificate, "other", "write"));
}

#[tokio::test]
async fn test_certify_verify_expired() {
    let authority = pair().await.unwrap();
    let expired_time = chrono::Utc::now().timestamp_millis() as f64 - 3600000.0; // 1 hour ago (expired)
    
    let cert = certify(
        Certificants::Wildcard,
        Policy::String("test".to_string()),
        &authority,
        CertifyOptions {
            expiry: Some(expired_time),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    use gun::sea::{verify_certificate, check_permission};
    let certificate = verify_certificate(&cert, &authority.pub_key).await.unwrap();
    
    // Check permission should fail for expired certificate
    assert!(!check_permission(&certificate, "test", "write"));
}
