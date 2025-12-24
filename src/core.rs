use crate::graph::Graph;
use crate::state::State;
use crate::events::EventEmitter;
use crate::storage::Storage;
use crate::dup::Dup;
use std::sync::Arc;

/// Core Gun instance structure
/// Based on Gun.js root.js and core.js
pub struct GunCore {
    pub graph: Arc<Graph>,
    pub state: Arc<State>,
    pub events: Arc<EventEmitter>,
    pub storage: Option<Arc<dyn Storage>>,
    pub id_counter: Arc<std::sync::atomic::AtomicU64>,
    pub dup: Arc<tokio::sync::RwLock<Dup>>, // Message deduplication for DAM
}

impl GunCore {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(Graph::new()),
            state: Arc::new(State::new()),
            events: Arc::new(EventEmitter::new()),
            storage: None,
            id_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            dup: Arc::new(tokio::sync::RwLock::new(Dup::default())),
        }
    }

    pub fn with_storage(storage: Arc<dyn Storage>) -> Self {
        Self {
            graph: Arc::new(Graph::new()),
            state: Arc::new(State::new()),
            events: Arc::new(EventEmitter::new()),
            storage: Some(storage),
            id_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            dup: Arc::new(tokio::sync::RwLock::new(Dup::default())),
        }
    }

    /// Generate a new soul (UUID) for a node
    /// Based on Gun.js uuid generation: Gun.state().toString(36).replace('.','') + String.random(12)
    pub fn uuid(&self, length: Option<usize>) -> String {
        let len = length.unwrap_or(12);
        let state = self.state.next();
        
        // Convert state to base36 string (similar to JavaScript toString(36))
        let state_str = format!("{:x}", state as u64).replace(".", "");
        
        // Generate random alphanumeric string
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars().collect();
        let random_part: String = (0..len)
            .map(|_| {
                let idx = rng.gen_range(0..chars.len());
                chars[idx]
            })
            .collect();
        
        format!("{}{}", state_str, random_part)
    }

    /// Generate a simple UUID (just random, for message IDs, etc.)
    pub fn random_id(&self, length: usize) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars().collect();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..chars.len());
                chars[idx]
            })
            .collect()
    }

    /// Get next chain ID
    pub fn next_chain_id(&self) -> u64 {
        self.id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for GunCore {
    fn default() -> Self {
        Self::new()
    }
}

