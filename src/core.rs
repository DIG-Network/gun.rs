use crate::dup::Dup;
use crate::events::EventEmitter;
use crate::graph::Graph;
use crate::state::State;
use crate::storage::Storage;
use std::sync::Arc;

/// Core Gun instance structure
///
/// This is the central engine that powers all Gun operations. It manages:
/// - **Graph**: In-memory storage of all nodes
/// - **State**: Timestamp generation for conflict resolution
/// - **Events**: Event system for reactive updates
/// - **Storage**: Optional persistent storage backend
/// - **Dedup**: Message deduplication for network operations
///
/// Based on Gun.js `root.js` and `core.js`. This is an internal structure
/// that is wrapped by the public [`Gun`](crate::Gun) type.
///
/// # Example
///
/// ```rust,no_run
/// use gun::core::GunCore;
///
/// let core = GunCore::new();
/// let soul = core.uuid(None);
/// println!("Generated soul: {}", soul);
/// ```
pub struct GunCore {
    pub graph: Arc<Graph>,
    pub state: Arc<State>,
    pub events: Arc<EventEmitter>,
    pub storage: Option<Arc<dyn Storage>>,
    pub id_counter: Arc<std::sync::atomic::AtomicU64>,
    pub dup: Arc<tokio::sync::RwLock<Dup>>, // Message deduplication for DAM
}

impl GunCore {
    /// Create a new GunCore instance without persistent storage
    ///
    /// This creates an in-memory only instance. Use [`with_storage`](Self::with_storage)
    /// to enable persistent storage.
    ///
    /// # Returns
    /// A new `GunCore` instance with no persistent storage.
    pub fn new() -> Self {
        Self {
            graph: Arc::new(Graph::new()),
            state: Arc::new(State::new()),
            events: Arc::new(EventEmitter::new()),
            storage: None,
            id_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            dup: Arc::new(tokio::sync::RwLock::new(Dup::new_default())),
        }
    }

    /// Create a new GunCore instance with persistent storage
    ///
    /// This enables data persistence across application restarts. The storage
    /// backend can be any implementation of the [`Storage`](crate::storage::Storage) trait,
    /// such as [`LocalStorage`](crate::storage::LocalStorage) or [`SledStorage`](crate::storage::SledStorage).
    ///
    /// # Arguments
    /// * `storage` - Storage backend implementing the `Storage` trait
    ///
    /// # Returns
    /// A new `GunCore` instance with persistent storage enabled.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gun::core::GunCore;
    /// use gun::storage::LocalStorage;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let storage = Arc::new(LocalStorage::new("./gun_data")?);
    /// let core = GunCore::with_storage(storage);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_storage(storage: Arc<dyn Storage>) -> Self {
        Self {
            graph: Arc::new(Graph::new()),
            state: Arc::new(State::new()),
            events: Arc::new(EventEmitter::new()),
            storage: Some(storage),
            id_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            dup: Arc::new(tokio::sync::RwLock::new(Dup::new_default())),
        }
    }

    /// Generate a new soul (UUID) for a node
    ///
    /// Souls are unique identifiers for nodes in the Gun graph. They combine:
    /// - A state-based component derived from the current timestamp
    /// - A random component for uniqueness
    ///
    /// Based on Gun.js uuid generation: `Gun.state().toString(36).replace('.','') + String.random(12)`
    ///
    /// # Arguments
    /// * `length` - Optional length of the random component (default: 12)
    ///
    /// # Returns
    /// A unique soul string suitable for use as a node identifier.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gun::core::GunCore;
    ///
    /// let core = GunCore::new();
    /// let soul = core.uuid(None); // Default 12-character random suffix
    /// let short_soul = core.uuid(Some(6)); // 6-character random suffix
    /// ```
    pub fn uuid(&self, length: Option<usize>) -> String {
        let len = length.unwrap_or(12);
        let state = self.state.next();

        // Convert state to base36 string (similar to JavaScript toString(36))
        let state_str = format!("{:x}", state as u64).replace(".", "");

        // Generate random alphanumeric string
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
            .chars()
            .collect();
        let random_part: String = (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..chars.len());
                chars[idx]
            })
            .collect();

        format!("{}{}", state_str, random_part)
    }

    /// Generate a simple random ID (for message IDs, etc.)
    ///
    /// This generates a pure random alphanumeric string without the state component.
    /// Useful for message IDs and other temporary identifiers.
    ///
    /// # Arguments
    /// * `length` - Length of the random ID to generate
    ///
    /// # Returns
    /// A random alphanumeric string of the specified length.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gun::core::GunCore;
    ///
    /// let core = GunCore::new();
    /// let message_id = core.random_id(16);
    /// ```
    pub fn random_id(&self, length: usize) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
            .chars()
            .collect();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..chars.len());
                chars[idx]
            })
            .collect()
    }

    /// Get the next unique chain ID
    ///
    /// Chain IDs are used internally to track chain instances for listener management.
    /// This increments atomically and returns a unique identifier for each chain.
    ///
    /// # Returns
    /// A unique chain ID (monotonically increasing counter).
    pub fn next_chain_id(&self) -> u64 {
        self.id_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for GunCore {
    fn default() -> Self {
        Self::new()
    }
}
