use async_trait::async_trait;
use serde_json::Value;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs;
use std::io::{Write, Read};
use crate::error::{GunResult, GunError};
use crate::state::Node;

/// Storage trait - pluggable storage backends
/// Based on Gun.js storage adapters (localStorage, RAD, S3, etc.)
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get a node by soul
    async fn get(&self, soul: &str) -> GunResult<Option<Node>>;
    
    /// Put a node (save)
    async fn put(&self, soul: &str, node: &Node) -> GunResult<()>;
    
    /// Check if a node exists
    async fn has(&self, soul: &str) -> GunResult<bool>;
}

/// In-memory storage (no persistence)
/// Useful for testing or temporary data
pub struct MemoryStorage {
    data: RwLock<HashMap<String, Node>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, soul: &str) -> GunResult<Option<Node>> {
        let data = self.data.read();
        Ok(data.get(soul).cloned())
    }

    async fn put(&self, soul: &str, node: &Node) -> GunResult<()> {
        let mut data = self.data.write();
        data.insert(soul.to_string(), node.clone());
        Ok(())
    }

    async fn has(&self, soul: &str) -> GunResult<bool> {
        let data = self.data.read();
        Ok(data.contains_key(soul))
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Sled-based persistent storage
/// Uses sled embedded database for persistence
pub struct SledStorage {
    db: sled::Db,
}

impl SledStorage {
    pub fn new(path: &str) -> GunResult<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
}

#[async_trait]
impl Storage for SledStorage {
    async fn get(&self, soul: &str) -> GunResult<Option<Node>> {
        match self.db.get(soul)? {
            Some(ivec) => {
                let json_str = String::from_utf8(ivec.to_vec())
                    .map_err(|e| GunError::InvalidData(format!("Invalid UTF-8: {}", e)))?;
                let node: Node = serde_json::from_str(&json_str)?;
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    async fn put(&self, soul: &str, node: &Node) -> GunResult<()> {
        let json_str = serde_json::to_string(node)?;
        self.db.insert(soul, json_str.as_bytes())?;
        self.db.flush_async().await?;
        Ok(())
    }

    async fn has(&self, soul: &str) -> GunResult<bool> {
        Ok(self.db.contains_key(soul)?)
    }
}

/// LocalStorage-equivalent storage for Rust
/// Provides a simple, persistent key-value store similar to browser localStorage
/// Stores data in JSON files on disk in a single directory
/// 
/// This is similar to browser localStorage in that it:
/// - Persists data to disk
/// - Provides simple get/put/has operations
/// - Stores data in a user-accessible location
/// - Is simpler than a full database (like Sled)
pub struct LocalStorage {
    data_dir: PathBuf,
    cache: RwLock<HashMap<String, Node>>, // In-memory cache for performance
    dirty: RwLock<HashSet<String>>, // Track which keys need to be written to disk
}

impl LocalStorage {
    /// Create a new LocalStorage instance
    /// 
    /// # Arguments
    /// * `data_dir` - Directory path where data will be stored (e.g., "./gun_data")
    /// 
    /// Creates the directory if it doesn't exist
    pub fn new(data_dir: &str) -> GunResult<Self> {
        let path = PathBuf::from(data_dir);
        
        // Create directory if it doesn't exist
        fs::create_dir_all(&path)
            .map_err(|e| GunError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create storage directory: {}", e)
            )))?;
        
        // Load existing data into cache
        let cache = Self::load_all(&path)?;
        
        Ok(Self {
            data_dir: path,
            cache: RwLock::new(cache),
            dirty: RwLock::new(HashSet::new()),
        })
    }
    
    /// Load all data from disk into memory cache
    fn load_all(path: &PathBuf) -> GunResult<HashMap<String, Node>> {
        let mut data = HashMap::new();
        
        // Read all files in the directory
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_path = entry.path();
                    if file_path.is_file() {
                        if let Some(file_name) = file_path.file_name() {
                            if let Some(soul) = file_name.to_str() {
                                // Try to decode the filename (may be URL-encoded)
                                let soul = urlencoding::decode(soul)
                                    .unwrap_or(std::borrow::Cow::Borrowed(soul))
                                    .into_owned();
                                
                                if let Ok(node) = Self::load_file(&file_path) {
                                    data.insert(soul, node);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(data)
    }
    
    /// Load a single file from disk
    fn load_file(path: &PathBuf) -> GunResult<Node> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let node: Node = serde_json::from_str(&contents)?;
        Ok(node)
    }
    
    /// Save a node to disk
    fn save_file(&self, soul: &str, node: &Node) -> GunResult<()> {
        // Encode soul as filename-safe (URL encoding)
        let encoded_soul = urlencoding::encode(soul);
        let file_path = self.data_dir.join(encoded_soul.as_ref());
        
        let json_str = serde_json::to_string_pretty(node)
            .map_err(|e| GunError::Serialization(e))?;
        
        // Write atomically: write to temp file, then rename
        let temp_path = file_path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(json_str.as_bytes())?;
        file.sync_all()?;
        drop(file);
        
        // Atomic rename
        fs::rename(&temp_path, &file_path)?;
        
        Ok(())
    }
    
    /// Flush dirty entries to disk
    pub async fn flush(&self) -> GunResult<()> {
        let dirty_keys: Vec<String> = {
            let dirty = self.dirty.read();
            dirty.iter().cloned().collect()
        };
        
        let cache = self.cache.read();
        for soul in dirty_keys {
            if let Some(node) = cache.get(&soul) {
                if let Err(e) = self.save_file(&soul, node) {
                    eprintln!("Error saving {} to disk: {}", soul, e);
                }
            }
        }
        
        // Clear dirty set
        let mut dirty = self.dirty.write();
        dirty.clear();
        
        Ok(())
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn get(&self, soul: &str) -> GunResult<Option<Node>> {
        // Check cache first
        let cache = self.cache.read();
        Ok(cache.get(soul).cloned())
    }

    async fn put(&self, soul: &str, node: &Node) -> GunResult<()> {
        // Update cache
        {
            let mut cache = self.cache.write();
            cache.insert(soul.to_string(), node.clone());
        }
        
        // Mark as dirty for disk write
        {
            let mut dirty = self.dirty.write();
            dirty.insert(soul.to_string());
        }
        
        // Write to disk immediately (localStorage behavior)
        // Could be optimized to batch writes, but for now we match localStorage's synchronous behavior
        self.save_file(soul, node)?;
        
        // Remove from dirty set since we just wrote it
        let mut dirty = self.dirty.write();
        dirty.remove(soul);
        
        Ok(())
    }

    async fn has(&self, soul: &str) -> GunResult<bool> {
        let cache = self.cache.read();
        Ok(cache.contains_key(soul))
    }
}

// Implement Drop to flush on cleanup
impl Drop for LocalStorage {
    fn drop(&mut self) {
        // Flush any remaining dirty entries
        let dirty_keys: Vec<String> = {
            let dirty = self.dirty.read();
            dirty.iter().cloned().collect()
        };
        
        if !dirty_keys.is_empty() {
            let cache = self.cache.read();
            for soul in dirty_keys {
                if let Some(node) = cache.get(&soul) {
                    let _ = self.save_file(&soul, node);
                }
            }
        }
    }
}
