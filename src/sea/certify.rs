//! Certificate-Based Access Control
//! Based on Gun.js sea/certify.js
//! Allows an authority to grant permissions to certificants for specific paths/patterns

use super::{sign, KeyPair, SeaError};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Certificate structure matching Gun.js format
/// Contains certificants, policies (read/write), expiry, and blocks
#[derive(Clone, Debug)]
pub struct Certificate {
    /// Certificants who have the rights (can be "*" for wildcard or list of pub keys)
    pub certificants: Certificants,
    /// Expiry timestamp (optional)
    pub expiry: Option<f64>,
    /// Read policy (optional)
    pub read_policy: Option<Policy>,
    /// Write policy (optional)
    pub write_policy: Option<Policy>,
    /// Read block/blacklist (optional)
    pub read_block: Option<String>,
    /// Write block/blacklist (optional)
    pub write_block: Option<String>,
}

/// Certificants can be wildcard or a list of public keys
#[derive(Clone, Debug)]
pub enum Certificants {
    /// Wildcard: anyone can have rights
    Wildcard,
    /// List of public keys
    List(Vec<String>),
}

/// Policy for access control
/// Can be a string, RAD/LEX object, or array of policies
#[derive(Clone, Debug)]
pub enum Policy {
    /// Simple string policy (e.g., "inbox")
    String(String),
    /// RAD/LEX object with pattern matching
    Radix(RadixPolicy),
    /// Array of policies
    Array(Vec<Policy>),
}

/// RAD/LEX pattern matching policy
/// Supports patterns like '*', '>', '<', '?', etc.
#[derive(Clone, Debug)]
pub struct RadixPolicy {
    /// Pattern map (key -> pattern)
    pub patterns: HashMap<String, String>,
}

/// Options for certificate creation
#[derive(Clone, Debug, Default)]
pub struct CertifyOptions {
    /// Expiry timestamp (optional)
    pub expiry: Option<f64>,
    /// Block/blacklist (optional)
    pub block: Option<BlockPolicy>,
    /// If true, return raw certificate without "SEA" prefix
    pub raw: bool,
}

/// Block/blacklist policy
#[derive(Clone, Debug)]
pub struct BlockPolicy {
    /// Read block
    pub read: Option<String>,
    /// Write block
    pub write: Option<String>,
}


/// Create a certificate
/// Based on Gun.js SEA.certify()
///
/// # Arguments
/// * `certificants` - Who gets the rights: "*", a pub key string, or Vec of pub keys
/// * `policy` - Access policy: string, RAD/LEX object, or array
/// * `authority` - Key pair of the certificate authority
/// * `opt` - Certificate options
///
/// # Returns
/// Signed certificate string (with "SEA" prefix unless raw=true)
///
/// # Examples
/// ```rust
/// use gun::sea::{certify, pair, CertifyOptions, Policy};
///
/// let authority = pair().await?;
/// let cert = certify(
///     "*", // anyone
///     Policy::String("inbox".to_string()),
///     &authority,
///     CertifyOptions::default()
/// ).await?;
/// ```
pub async fn certify(
    certificants: Certificants,
    policy: Policy,
    authority: &KeyPair,
    opt: CertifyOptions,
) -> Result<String, SeaError> {
    // Normalize certificants
    let certs = match certificants {
        Certificants::Wildcard => "*".to_string(),
        Certificants::List(list) => {
            if list.len() == 1 {
                list[0].clone()
            } else {
                serde_json::to_string(&list)
                    .map_err(|e| SeaError::Crypto(format!("Serialization error: {}", e)))?
            }
        }
    };

    // Extract read/write policies
    // In Gun.js, if policy is an object with 'read' key, use it; otherwise assume write
    let (read_policy, write_policy): (Option<String>, Option<String>) = match policy {
        Policy::String(s) => (None, Some(s)),
        Policy::Radix(r) => {
            // For RAD/LEX, serialize patterns as JSON
            (
                None,
                Some(serde_json::to_string(&r.patterns).unwrap_or_default()),
            )
        }
        Policy::Array(policies) => {
            // Arrays are complex, convert to JSON array
            let policy_strs: Vec<String> = policies.iter().map(|p| format!("{:?}", p)).collect();
            (
                None,
                Some(serde_json::to_string(&policy_strs).unwrap_or_default()),
            )
        }
    };

    // Build certificate data (matching Gun.js format)
    let mut cert_data = json!({
        "c": certs, // certificants
    });

    if let Some(expiry) = opt.expiry {
        cert_data["e"] = json!(expiry);
    }

    if let Some(ref read) = read_policy {
        cert_data["r"] = json!(read);
    }

    if let Some(ref write) = write_policy {
        cert_data["w"] = json!(write);
    }

    if let Some(ref block) = opt.block {
        if let Some(ref read_block) = block.read {
            cert_data["rb"] = json!(read_block);
        }
        if let Some(ref write_block) = block.write {
            cert_data["wb"] = json!(write_block);
        }
    }

    // Sign the certificate (sign expects Value, not string)
    let signature = sign(&cert_data, authority).await?;

    // Format: "SEA{...}" unless raw (matching Gun.js format)
    if opt.raw {
        // Return raw signature object
        serde_json::to_string(&signature)
            .map_err(|e| SeaError::Crypto(format!("Serialization error: {}", e)))
    } else {
        // Wrap in "SEA" prefix
        let sig_str = serde_json::to_string(&signature)
            .map_err(|e| SeaError::Crypto(format!("Serialization error: {}", e)))?;
        Ok(format!("SEA{}", sig_str))
    }
}

/// Verify a certificate with full signature verification
/// 
/// Verifies a certificate's signature using the authority's public key and extracts
/// the certificate data. This performs full cryptographic verification, not just JSON parsing.
/// 
/// # Arguments
/// * `cert` - Certificate string (with optional "SEA" prefix)
/// * `authority_pub` - Public key of the certificate authority in base64 x.y format
/// 
/// # Returns
/// `Certificate` struct with verified data if signature is valid
/// 
/// # Errors
/// - `SeaError::Crypto`: If certificate format is invalid
/// - `SeaError::VerificationFailed`: If signature verification fails (wrong authority or tampered data)
/// 
/// # Security
/// 
/// This function performs full cryptographic signature verification using ECDSA P-256.
/// Certificates with invalid signatures or wrong authority keys will be rejected.
/// 
/// # Example
/// ```rust,no_run
/// use gun::sea::{certify, verify_certificate, pair, Certificants, Policy, CertifyOptions};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let authority = pair().await?;
/// 
/// // Create certificate
/// let cert = certify(
///     Certificants::Wildcard,
///     Policy::String("inbox".to_string()),
///     &authority,
///     CertifyOptions::default(),
/// ).await?;
/// 
/// // Verify certificate
/// let verified = verify_certificate(&cert, &authority.pub_key).await?;
/// println!("Certificate verified for: {:?}", verified.certificants);
/// # Ok(())
/// # }
/// ```
pub async fn verify_certificate(cert: &str, authority_pub: &str) -> Result<Certificate, SeaError> {
    // Remove "SEA" prefix if present
    let cert_data = if let Some(stripped) = cert.strip_prefix("SEA{") {
        // Remove "SEA{" prefix and find matching "}"
        // The certificate is a JSON object, so we need to parse it properly
        let mut json_str = String::from("{");
        json_str.push_str(stripped);
        json_str
    } else if let Some(stripped) = cert.strip_prefix("SEA") {
        stripped.to_string()
    } else {
        cert.to_string()
    };

    // Parse certificate as signed data (format: {m: message, s: signature})
    let signed_data: Value = serde_json::from_str(&cert_data)
        .map_err(|e| SeaError::Crypto(format!("Parse error: {}", e)))?;

    // Verify signature using SEA.verify()
    use super::verify;
    let parsed = verify(&signed_data, authority_pub).await?;

    // Extract certificate fields
    let certificants = if let Some(c) = parsed.get("c") {
        if let Some(s) = c.as_str() {
            if s == "*" {
                Certificants::Wildcard
            } else {
                Certificants::List(vec![s.to_string()])
            }
        } else if let Some(arr) = c.as_array() {
            Certificants::List(
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
            )
        } else {
            return Err(SeaError::Crypto("Invalid certificants format".to_string()));
        }
    } else {
        return Err(SeaError::Crypto("Missing certificants".to_string()));
    };

    let expiry = parsed.get("e").and_then(|v| v.as_f64());
    let read_policy = parsed
        .get("r")
        .and_then(|v| v.as_str().map(|s| Policy::String(s.to_string())));
    let write_policy = parsed
        .get("w")
        .and_then(|v| v.as_str().map(|s| Policy::String(s.to_string())));
    let read_block = parsed
        .get("rb")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let write_block = parsed
        .get("wb")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    Ok(Certificate {
        certificants,
        expiry,
        read_policy,
        write_policy,
        read_block,
        write_block,
    })
}

/// Check if a path matches a policy
/// Implements RAD/LEX pattern matching
pub fn matches_policy(path: &str, policy: &Policy) -> bool {
    match policy {
        Policy::String(s) => {
            // Simple string match
            path == s || path.starts_with(&format!("{}/", s))
        }
        Policy::Radix(r) => {
            // RAD/LEX pattern matching
            // For now, simple implementation - full RAD/LEX would need more complex matching
            for pattern in r.patterns.keys() {
                if pattern == "*" {
                    return true; // Wildcard matches everything
                }
                if path.starts_with(pattern) {
                    return true;
                }
            }
            false
        }
        Policy::Array(policies) => {
            // Array: any policy matches
            policies.iter().any(|p| matches_policy(path, p))
        }
    }
}

/// Check if a certificate grants permission for a path
pub fn check_permission(
    cert: &Certificate,
    path: &str,
    operation: &str, // "read" or "write"
) -> bool {
    // Check expiry
    if let Some(expiry) = cert.expiry {
        let now = chrono::Utc::now().timestamp_millis() as f64;
        if now > expiry {
            return false; // Expired
        }
    }

    // Check blocks
    if operation == "read" {
        if let Some(ref block) = cert.read_block {
            if path.contains(block) {
                return false; // Blocked
            }
        }
    } else if operation == "write" {
        if let Some(ref block) = cert.write_block {
            if path.contains(block) {
                return false; // Blocked
            }
        }
    }

    // Check policies
    if operation == "read" {
        if let Some(ref policy) = cert.read_policy {
            return matches_policy(path, policy);
        }
    } else if operation == "write" {
        if let Some(ref policy) = cert.write_policy {
            return matches_policy(path, policy);
        }
    }

    false
}
