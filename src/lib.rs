use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use nodespace_core_types::{DatabaseError, Node, NodeId, NodeSpaceError, NodeSpaceResult, ProcessingError, ValidationError};
use nodespace_data_store::NodeType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// Import traits from their respective repositories
pub use nodespace_data_store::DataStore;
pub use nodespace_nlp_engine::NLPEngine;

// Import additional types for embedding generation bridge
use nodespace_data_store::DataStoreError;
use nodespace_data_store::EmbeddingGenerator as DataStoreEmbeddingGenerator;

/// Adapter that bridges NLPEngine to DataStore's EmbeddingGenerator trait
/// This allows the data store to automatically generate embeddings using the NLP engine
pub struct NLPEmbeddingAdapter<N: NLPEngine> {
    nlp_engine: N,
}

impl<N: NLPEngine> NLPEmbeddingAdapter<N> {
    pub fn new(nlp_engine: N) -> Self {
        Self { nlp_engine }
    }
}

#[async_trait]
impl<N: NLPEngine + Send + Sync> DataStoreEmbeddingGenerator for NLPEmbeddingAdapter<N> {
    async fn generate_embedding(&self, content: &str) -> Result<Vec<f32>, DataStoreError> {
        // Bridge to the NLPEngine trait implementation
        match self.nlp_engine.generate_embedding(content).await {
            Ok(embedding) => Ok(embedding),
            Err(e) => Err(DataStoreError::EmbeddingError(format!(
                "NLP engine embedding failed: {}",
                e
            ))),
        }
    }
}

/// Smart embedding cache with dependency invalidation for enhanced RAG performance
pub mod smart_embedding_cache {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    /// Content hash for cache keys
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ContentHash(pub String);

    impl ContentHash {
        pub fn from_content(content: &str) -> Self {
            let mut hasher = DefaultHasher::new();
            content.hash(&mut hasher);
            Self(format!("{:x}", hasher.finish()))
        }
    }

    /// Context hash for contextual embeddings
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ContextHash {
        pub content_hash: ContentHash,
        pub parent_hash: Option<ContentHash>,
        pub sibling_hashes: Vec<ContentHash>,
        pub mention_hashes: Vec<ContentHash>,
        pub strategy: ContextStrategy,
    }

    /// Hierarchical path hash for hierarchical embeddings
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct PathHash {
        pub content_hash: ContentHash,
        pub path_hashes: Vec<ContentHash>, // From root to this node
    }

    /// Context strategy for embedding generation
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum ContextStrategy {
        RuleBased,    // Fast rule-based context generation
        Phi4Enhanced, // Phi-4 curated context (future)
        Adaptive,     // Choose strategy based on content
    }

    /// Relationship fingerprint for tracking changes
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RelationshipFingerprint {
        pub parent_id: Option<NodeId>,
        pub sibling_ids: Vec<NodeId>,
        pub mention_ids: Vec<NodeId>,
        pub last_modified: DateTime<Utc>,
    }

    /// Cache entry with metadata
    #[derive(Debug, Clone)]
    pub struct CacheEntry {
        pub embedding: Vec<f32>,
        pub created_at: Instant,
        pub last_accessed: Instant,
        pub access_count: u64,
        pub fingerprint: Option<RelationshipFingerprint>,
    }

    impl CacheEntry {
        pub fn new(embedding: Vec<f32>, fingerprint: Option<RelationshipFingerprint>) -> Self {
            let now = Instant::now();
            Self {
                embedding,
                created_at: now,
                last_accessed: now,
                access_count: 1,
                fingerprint,
            }
        }

        pub fn access(&mut self) {
            self.last_accessed = Instant::now();
            self.access_count += 1;
        }
    }

    /// LRU cache implementation for embeddings
    #[derive(Debug)]
    pub struct LruCache<K: Clone + Eq + Hash, V> {
        map: HashMap<K, V>,
        access_order: Vec<K>,
        capacity: usize,
    }

    impl<K: Clone + Eq + Hash, V> LruCache<K, V> {
        pub fn new(capacity: usize) -> Self {
            Self {
                map: HashMap::new(),
                access_order: Vec::new(),
                capacity,
            }
        }

        pub fn get(&mut self, key: &K) -> Option<&mut V> {
            if self.map.contains_key(key) {
                // Move to end (most recently used)
                if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                    let key = self.access_order.remove(pos);
                    self.access_order.push(key);
                }
                self.map.get_mut(key)
            } else {
                None
            }
        }

        pub fn insert(&mut self, key: K, value: V) {
            if self.map.contains_key(&key) {
                // Update existing
                self.map.insert(key.clone(), value);
                // Move to end
                if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                    let key = self.access_order.remove(pos);
                    self.access_order.push(key);
                }
            } else {
                // Insert new
                if self.map.len() >= self.capacity {
                    // Evict least recently used
                    if let Some(oldest_key) = self.access_order.first().cloned() {
                        self.map.remove(&oldest_key);
                        self.access_order.remove(0);
                    }
                }
                self.map.insert(key.clone(), value);
                self.access_order.push(key);
            }
        }

        pub fn remove(&mut self, key: &K) -> Option<V> {
            if let Some(value) = self.map.remove(key) {
                if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                    self.access_order.remove(pos);
                }
                Some(value)
            } else {
                None
            }
        }

        pub fn clear(&mut self) {
            self.map.clear();
            self.access_order.clear();
        }

        pub fn len(&self) -> usize {
            self.map.len()
        }

        pub fn is_empty(&self) -> bool {
            self.map.is_empty()
        }

        pub fn capacity(&self) -> usize {
            self.capacity
        }
    }

    /// Cache performance metrics
    #[derive(Debug, Default, Clone)]
    pub struct CacheMetrics {
        pub individual_hits: u64,
        pub individual_misses: u64,
        pub contextual_hits: u64,
        pub contextual_misses: u64,
        pub hierarchical_hits: u64,
        pub hierarchical_misses: u64,
        pub invalidations: u64,
        pub memory_usage_bytes: usize,
        pub last_reset: Option<Instant>,
    }

    impl CacheMetrics {
        pub fn new() -> Self {
            Self {
                last_reset: Some(Instant::now()),
                ..Default::default()
            }
        }

        pub fn individual_hit_rate(&self) -> f64 {
            let total = self.individual_hits + self.individual_misses;
            if total > 0 {
                self.individual_hits as f64 / total as f64
            } else {
                0.0
            }
        }

        pub fn contextual_hit_rate(&self) -> f64 {
            let total = self.contextual_hits + self.contextual_misses;
            if total > 0 {
                self.contextual_hits as f64 / total as f64
            } else {
                0.0
            }
        }

        pub fn hierarchical_hit_rate(&self) -> f64 {
            let total = self.hierarchical_hits + self.hierarchical_misses;
            if total > 0 {
                self.hierarchical_hits as f64 / total as f64
            } else {
                0.0
            }
        }

        pub fn overall_hit_rate(&self) -> f64 {
            let total_hits = self.individual_hits + self.contextual_hits + self.hierarchical_hits;
            let total_misses =
                self.individual_misses + self.contextual_misses + self.hierarchical_misses;
            let total = total_hits + total_misses;
            if total > 0 {
                total_hits as f64 / total as f64
            } else {
                0.0
            }
        }

        pub fn reset(&mut self) {
            *self = Self::new();
        }
    }

    /// Smart embedding cache with multi-tier architecture and dependency tracking
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct SmartEmbeddingCache {
        // Tiered cache storage
        individual_cache: LruCache<ContentHash, CacheEntry>,
        contextual_cache: LruCache<ContextHash, CacheEntry>,
        hierarchical_cache: LruCache<PathHash, CacheEntry>,

        // Dependency tracking
        dependency_graph: HashMap<NodeId, HashSet<NodeId>>, // Who depends on this node
        relationship_fingerprints: HashMap<NodeId, RelationshipFingerprint>,

        // Performance monitoring
        metrics: CacheMetrics,

        // Configuration
        max_individual_entries: usize,
        max_contextual_entries: usize,
        max_hierarchical_entries: usize,
        cache_ttl: Duration,
    }

    impl Default for SmartEmbeddingCache {
        fn default() -> Self {
            Self::new()
        }
    }

    #[allow(dead_code)]
    impl SmartEmbeddingCache {
        pub fn new() -> Self {
            Self::with_capacity(10000, 5000, 2000)
        }

        pub fn with_capacity(
            individual_capacity: usize,
            contextual_capacity: usize,
            hierarchical_capacity: usize,
        ) -> Self {
            Self {
                individual_cache: LruCache::new(individual_capacity),
                contextual_cache: LruCache::new(contextual_capacity),
                hierarchical_cache: LruCache::new(hierarchical_capacity),
                dependency_graph: HashMap::new(),
                relationship_fingerprints: HashMap::new(),
                metrics: CacheMetrics::new(),
                max_individual_entries: individual_capacity,
                max_contextual_entries: contextual_capacity,
                max_hierarchical_entries: hierarchical_capacity,
                cache_ttl: Duration::from_secs(3600), // 1 hour TTL
            }
        }

        /// Get individual embedding from cache
        pub fn get_individual_embedding(&mut self, content_hash: &ContentHash) -> Option<Vec<f32>> {
            if let Some(entry) = self.individual_cache.get(content_hash) {
                entry.access();
                self.metrics.individual_hits += 1;
                Some(entry.embedding.clone())
            } else {
                self.metrics.individual_misses += 1;
                None
            }
        }

        /// Cache individual embedding
        pub fn cache_individual_embedding(
            &mut self,
            content_hash: ContentHash,
            embedding: Vec<f32>,
        ) {
            let entry = CacheEntry::new(embedding, None);
            self.individual_cache.insert(content_hash, entry);
            self.update_memory_usage();
        }

        /// Get contextual embedding from cache
        pub fn get_contextual_embedding(&mut self, context_hash: &ContextHash) -> Option<Vec<f32>> {
            // First, check if the entry exists
            let has_entry = self.contextual_cache.map.contains_key(context_hash);

            if !has_entry {
                self.metrics.contextual_misses += 1;
                return None;
            }

            // Check validity and access the entry
            let embedding = if let Some(entry) = self.contextual_cache.get(context_hash) {
                // Create a temporary clone to check validity without borrowing self
                let is_expired = entry.created_at.elapsed() > self.cache_ttl;

                if is_expired {
                    // Entry is expired, will be removed
                    None
                } else {
                    // Entry is valid, access it and return embedding
                    entry.access();
                    Some(entry.embedding.clone())
                }
            } else {
                None
            };

            match embedding {
                Some(emb) => {
                    self.metrics.contextual_hits += 1;
                    Some(emb)
                }
                None => {
                    // Entry is stale, remove it
                    self.contextual_cache.remove(context_hash);
                    self.metrics.contextual_misses += 1;
                    None
                }
            }
        }

        /// Cache contextual embedding with relationship fingerprint
        pub fn cache_contextual_embedding(
            &mut self,
            context_hash: ContextHash,
            embedding: Vec<f32>,
            fingerprint: RelationshipFingerprint,
        ) {
            let entry = CacheEntry::new(embedding, Some(fingerprint));
            self.contextual_cache.insert(context_hash, entry);
            self.update_memory_usage();
        }

        /// Get hierarchical embedding from cache
        pub fn get_hierarchical_embedding(&mut self, path_hash: &PathHash) -> Option<Vec<f32>> {
            // First, check if the entry exists
            let has_entry = self.hierarchical_cache.map.contains_key(path_hash);

            if !has_entry {
                self.metrics.hierarchical_misses += 1;
                return None;
            }

            // Check validity and access the entry
            let embedding = if let Some(entry) = self.hierarchical_cache.get(path_hash) {
                // Create a temporary clone to check validity without borrowing self
                let is_expired = entry.created_at.elapsed() > self.cache_ttl;

                if is_expired {
                    // Entry is expired, will be removed
                    None
                } else {
                    // Entry is valid, access it and return embedding
                    entry.access();
                    Some(entry.embedding.clone())
                }
            } else {
                None
            };

            match embedding {
                Some(emb) => {
                    self.metrics.hierarchical_hits += 1;
                    Some(emb)
                }
                None => {
                    // Entry is stale, remove it
                    self.hierarchical_cache.remove(path_hash);
                    self.metrics.hierarchical_misses += 1;
                    None
                }
            }
        }

        /// Cache hierarchical embedding
        pub fn cache_hierarchical_embedding(
            &mut self,
            path_hash: PathHash,
            embedding: Vec<f32>,
            fingerprint: RelationshipFingerprint,
        ) {
            let entry = CacheEntry::new(embedding, Some(fingerprint));
            self.hierarchical_cache.insert(path_hash, entry);
            self.update_memory_usage();
        }

        /// Add dependency relationship for cache invalidation
        pub fn add_dependency(&mut self, node_id: NodeId, depends_on: NodeId) {
            self.dependency_graph
                .entry(depends_on)
                .or_insert_with(HashSet::new)
                .insert(node_id);
        }

        /// Update relationship fingerprint for a node
        pub fn update_fingerprint(
            &mut self,
            node_id: NodeId,
            fingerprint: RelationshipFingerprint,
        ) {
            self.relationship_fingerprints.insert(node_id, fingerprint);
        }

        /// Invalidate embeddings when a node changes
        pub fn invalidate_node_embeddings(&mut self, node_id: &NodeId) {
            self.metrics.invalidations += 1;

            // Get all nodes that depend on this node
            let dependents = self
                .dependency_graph
                .get(node_id)
                .cloned()
                .unwrap_or_default();

            // Invalidate contextual and hierarchical caches for dependents
            self.invalidate_contextual_caches(&dependents);
            self.invalidate_hierarchical_caches(&dependents);

            // Update fingerprint timestamp for the changed node
            if let Some(fingerprint) = self.relationship_fingerprints.get_mut(node_id) {
                fingerprint.last_modified = Utc::now();
            }

            // Recursively invalidate dependent nodes
            for dependent in dependents {
                self.invalidate_node_embeddings(&dependent);
            }
        }

        /// Clear all caches
        pub fn clear_all(&mut self) {
            self.individual_cache.clear();
            self.contextual_cache.clear();
            self.hierarchical_cache.clear();
            self.dependency_graph.clear();
            self.relationship_fingerprints.clear();
            self.metrics.reset();
        }

        /// Get cache metrics
        pub fn metrics(&self) -> &CacheMetrics {
            &self.metrics
        }

        /// Get cache statistics
        pub fn cache_stats(&self) -> CacheStats {
            CacheStats {
                individual_count: self.individual_cache.len(),
                individual_capacity: self.individual_cache.capacity(),
                contextual_count: self.contextual_cache.len(),
                contextual_capacity: self.contextual_cache.capacity(),
                hierarchical_count: self.hierarchical_cache.len(),
                hierarchical_capacity: self.hierarchical_cache.capacity(),
                dependency_count: self.dependency_graph.len(),
                memory_usage_bytes: self.metrics.memory_usage_bytes,
                overall_hit_rate: self.metrics.overall_hit_rate(),
            }
        }

        // Private helper methods

        fn is_contextual_entry_valid(&self, entry: &CacheEntry) -> bool {
            // Check if entry has expired
            if entry.created_at.elapsed() > self.cache_ttl {
                return false;
            }

            // Check if relationships have changed
            if let Some(fingerprint) = &entry.fingerprint {
                // Compare with current fingerprint
                if let Some(current_fingerprint) =
                    self.relationship_fingerprints.get(&NodeId::new())
                {
                    return fingerprint.last_modified <= current_fingerprint.last_modified;
                }
            }

            true
        }

        fn is_hierarchical_entry_valid(&self, entry: &CacheEntry) -> bool {
            // Check if entry has expired
            if entry.created_at.elapsed() > self.cache_ttl {
                return false;
            }

            // Hierarchical embeddings are more sensitive to changes
            if let Some(fingerprint) = &entry.fingerprint {
                // Any parent or sibling change invalidates hierarchical embeddings
                if let Some(current_fingerprint) =
                    self.relationship_fingerprints.get(&NodeId::new())
                {
                    return fingerprint.parent_id == current_fingerprint.parent_id
                        && fingerprint.sibling_ids == current_fingerprint.sibling_ids;
                }
            }

            true
        }

        fn invalidate_contextual_caches(&mut self, node_ids: &HashSet<NodeId>) {
            // Remove contextual cache entries that depend on the changed nodes
            let keys_to_remove: Vec<ContextHash> = self
                .contextual_cache
                .map
                .keys()
                .filter(|context_hash| {
                    // Check if any of the context hashes are affected
                    node_ids.iter().any(|node_id| {
                        let content_hash = ContentHash::from_content(&node_id.to_string());
                        context_hash.parent_hash == Some(content_hash.clone())
                            || context_hash.sibling_hashes.contains(&content_hash)
                            || context_hash.mention_hashes.contains(&content_hash)
                    })
                })
                .cloned()
                .collect();

            for key in keys_to_remove {
                self.contextual_cache.remove(&key);
            }
        }

        fn invalidate_hierarchical_caches(&mut self, node_ids: &HashSet<NodeId>) {
            // Remove hierarchical cache entries that include the changed nodes in their path
            let keys_to_remove: Vec<PathHash> = self
                .hierarchical_cache
                .map
                .keys()
                .filter(|path_hash| {
                    // Check if any of the path hashes are affected
                    node_ids.iter().any(|node_id| {
                        let content_hash = ContentHash::from_content(&node_id.to_string());
                        path_hash.path_hashes.contains(&content_hash)
                    })
                })
                .cloned()
                .collect();

            for key in keys_to_remove {
                self.hierarchical_cache.remove(&key);
            }
        }

        fn update_memory_usage(&mut self) {
            // Estimate memory usage (simplified calculation)
            let individual_size = self.individual_cache.len() * 768 * 4; // Assume 768-dim f32 embeddings
            let contextual_size = self.contextual_cache.len() * 768 * 4;
            let hierarchical_size = self.hierarchical_cache.len() * 768 * 4;

            self.metrics.memory_usage_bytes = individual_size + contextual_size + hierarchical_size;
        }
    }

    /// Cache statistics for monitoring
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CacheStats {
        pub individual_count: usize,
        pub individual_capacity: usize,
        pub contextual_count: usize,
        pub contextual_capacity: usize,
        pub hierarchical_count: usize,
        pub hierarchical_capacity: usize,
        pub dependency_count: usize,
        pub memory_usage_bytes: usize,
        pub overall_hit_rate: f64,
    }
}

/// Simple performance monitoring (compatible with all dependencies)
pub mod monitoring {
    use std::time::Instant;

    /// Simple performance timer for operations
    pub struct OperationTimer {
        _operation_name: String,
        _start_time: Instant,
    }

    impl OperationTimer {
        pub fn new(operation_name: &str) -> Self {
            Self {
                _operation_name: operation_name.to_string(),
                _start_time: Instant::now(),
            }
        }

        /// Add metadata (no-op for compatibility)
        pub fn with_metadata(self, _key: String, _value: String) -> Self {
            self
        }

        /// Complete operation successfully (no-op for compatibility)
        pub fn complete_success(self) {
            // No-op for now to avoid dependency conflicts
        }

        /// Complete operation with error (no-op for compatibility)
        pub fn complete_error(self, _error: String) {
            // No-op for now to avoid dependency conflicts
        }
    }

    /// Simple performance monitor (stub for compatibility)
    #[derive(Debug, Default)]
    pub struct PerformanceMonitor;

    impl PerformanceMonitor {
        pub fn new() -> Self {
            Self
        }

        /// Start timing an operation
        pub fn start_operation(&self, operation_name: &str) -> OperationTimer {
            OperationTimer::new(operation_name)
        }
    }
}

/// Configuration constants for NodeSpace service
pub mod constants {
    /// Default embedding model
    pub const DEFAULT_EMBEDDING_MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";
    /// Default text generation model
    pub const DEFAULT_TEXT_MODEL: &str = "mistralai/Mistral-7B-Instruct-v0.1";
    /// Default download timeout in seconds
    pub const DEFAULT_DOWNLOAD_TIMEOUT: u64 = 300;
    /// Default maximum batch size for operations
    pub const DEFAULT_MAX_BATCH_SIZE: usize = 32;
    /// Default context window size
    pub const DEFAULT_CONTEXT_WINDOW: usize = 4096;
    /// Default temperature for text generation
    pub const DEFAULT_TEMPERATURE: f32 = 0.7;
    /// Default search limit for semantic search
    pub const DEFAULT_SEARCH_LIMIT: usize = 5;
    /// Default search limit for multi-strategy search
    pub const DEFAULT_MULTI_STRATEGY_LIMIT: usize = 20;
    /// Default maximum results per strategy
    pub const DEFAULT_MAX_RESULTS_PER_STRATEGY: usize = 10;
    /// Default final result limit
    pub const DEFAULT_FINAL_RESULT_LIMIT: usize = 10;
    /// Score decay factor for search results
    pub const SCORE_DECAY_FACTOR: f32 = 0.1;
    /// Minimum search score
    pub const MIN_SEARCH_SCORE: f32 = 0.1;
    /// Base confidence for queries with context
    pub const BASE_CONFIDENCE_WITH_CONTEXT: f32 = 0.8;
    /// Base confidence for queries without context
    pub const BASE_CONFIDENCE_NO_CONTEXT: f32 = 0.3;
    /// Confidence reduction factor for fallback responses
    pub const FALLBACK_CONFIDENCE_FACTOR: f32 = 0.5;
    /// Default embedding dimension (for fallback)
    pub const DEFAULT_EMBEDDING_DIMENSION: usize = 768;
    /// Reserved space for prompt structure in context window
    pub const PROMPT_STRUCTURE_RESERVE: usize = 200;
}

/// Configuration for NodeSpace service initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpaceConfig {
    /// Model download preferences
    pub model_config: ModelConfig,
    /// Performance tuning options
    pub performance_config: PerformanceConfig,
    /// Offline operation settings
    pub offline_config: OfflineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Preferred embedding model (default: all-MiniLM-L6-v2)
    pub embedding_model: Option<String>,
    /// Preferred text generation model
    pub text_model: Option<String>,
    /// Model download timeout in seconds
    pub download_timeout: Option<u64>,
    /// Local model cache directory
    pub cache_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum batch size for embedding generation
    pub max_batch_size: Option<usize>,
    /// Context window size for text generation
    pub context_window: Option<usize>,
    /// Temperature for text generation (0.0-1.0)
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    /// Allow offline operation mode
    pub enable_offline: bool,
    /// Fallback behavior when models unavailable
    pub offline_fallback: OfflineFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OfflineFallback {
    /// Return error when models unavailable
    Error,
    /// Return stub responses for testing
    Stub,
    /// Use cached responses if available
    Cache,
}

impl Default for NodeSpaceConfig {
    fn default() -> Self {
        Self {
            model_config: ModelConfig {
                embedding_model: Some(constants::DEFAULT_EMBEDDING_MODEL.to_string()),
                text_model: Some(constants::DEFAULT_TEXT_MODEL.to_string()),
                download_timeout: Some(constants::DEFAULT_DOWNLOAD_TIMEOUT),
                cache_dir: None, // Use system default
            },
            performance_config: PerformanceConfig {
                max_batch_size: Some(constants::DEFAULT_MAX_BATCH_SIZE),
                context_window: Some(constants::DEFAULT_CONTEXT_WINDOW),
                temperature: Some(constants::DEFAULT_TEMPERATURE),
            },
            offline_config: OfflineConfig {
                enable_offline: true,
                offline_fallback: OfflineFallback::Cache,
            },
        }
    }
}

/// Service initialization state
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    Uninitialized,
    Initializing,
    Ready,
    Failed(String),
}

/// Core business logic service that orchestrates NodeSpace functionality
/// using distributed contract ownership
pub struct NodeSpaceService<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> {
    data_store: D,
    nlp_engine: N,
    config: NodeSpaceConfig,
    state: Arc<RwLock<ServiceState>>,
    performance_monitor: monitoring::PerformanceMonitor,
    hierarchy_cache: Arc<RwLock<HierarchyCache>>,
    embedding_cache: Arc<RwLock<smart_embedding_cache::SmartEmbeddingCache>>,
}

impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> NodeSpaceService<D, N> {
    /// Create a new NodeSpace service with injected dependencies
    pub fn new(data_store: D, nlp_engine: N) -> Self {
        Self::with_config(data_store, nlp_engine, NodeSpaceConfig::default())
    }

    /// Create a new NodeSpace service with custom configuration
    pub fn with_config(data_store: D, nlp_engine: N, config: NodeSpaceConfig) -> Self {
        Self {
            data_store,
            nlp_engine,
            config,
            state: Arc::new(RwLock::new(ServiceState::Uninitialized)),
            performance_monitor: monitoring::PerformanceMonitor::new(),
            hierarchy_cache: Arc::new(RwLock::new(HierarchyCache::new())),
            embedding_cache: Arc::new(RwLock::new(
                smart_embedding_cache::SmartEmbeddingCache::new(),
            )),
        }
    }

    /// Get performance monitor for metrics access
    pub fn performance_monitor(&self) -> &monitoring::PerformanceMonitor {
        &self.performance_monitor
    }

    /// Get embedding cache statistics
    pub async fn embedding_cache_stats(&self) -> smart_embedding_cache::CacheStats {
        let cache = self.embedding_cache.read().await;
        cache.cache_stats()
    }

    /// Clear embedding cache
    pub async fn clear_embedding_cache(&self) {
        let mut cache = self.embedding_cache.write().await;
        cache.clear_all();
    }

    /// Get embedding with intelligent caching
    pub async fn get_cached_embedding(&self, content: &str) -> NodeSpaceResult<Vec<f32>> {
        let content_hash = smart_embedding_cache::ContentHash::from_content(content);

        // Try to get from individual cache first
        {
            let mut cache = self.embedding_cache.write().await;
            if let Some(cached_embedding) = cache.get_individual_embedding(&content_hash) {
                return Ok(cached_embedding);
            }
        }

        // Generate new embedding if not cached
        let embedding = self.nlp_engine.generate_embedding(content).await?;

        // Cache the new embedding
        {
            let mut cache = self.embedding_cache.write().await;
            cache.cache_individual_embedding(content_hash, embedding.clone());
        }

        Ok(embedding)
    }

    /// Get contextual embedding with relationship tracking
    pub async fn get_cached_contextual_embedding(
        &self,
        node: &Node,
        context_strategy: smart_embedding_cache::ContextStrategy,
    ) -> NodeSpaceResult<Vec<f32>> {
        // Build context hash from node relationships
        let content_hash =
            smart_embedding_cache::ContentHash::from_content(node.content.as_str().unwrap_or(""));

        let parent_hash = if let Some(parent_id) = &node.parent_id {
            if let Ok(Some(parent_node)) = self.data_store.get_node(parent_id).await {
                Some(smart_embedding_cache::ContentHash::from_content(
                    parent_node.content.as_str().unwrap_or(""),
                ))
            } else {
                None
            }
        } else {
            None
        };

        // Get sibling hashes
        let sibling_hashes = if let Some(parent_id) = &node.parent_id {
            let siblings = self.get_children(parent_id).await.unwrap_or_default();
            siblings
                .iter()
                .filter(|sibling| sibling.id != node.id)
                .map(|sibling| {
                    smart_embedding_cache::ContentHash::from_content(
                        sibling.content.as_str().unwrap_or(""),
                    )
                })
                .collect()
        } else {
            Vec::new()
        };

        let context_hash = smart_embedding_cache::ContextHash {
            content_hash: content_hash.clone(),
            parent_hash,
            sibling_hashes,
            mention_hashes: Vec::new(), // TODO: Extract mentions
            strategy: context_strategy,
        };

        // Try to get from contextual cache
        {
            let mut cache = self.embedding_cache.write().await;
            if let Some(cached_embedding) = cache.get_contextual_embedding(&context_hash) {
                return Ok(cached_embedding);
            }
        }

        // Generate contextual embedding
        let contextual_content = self.build_contextual_content(node).await?;
        let embedding = self
            .nlp_engine
            .generate_embedding(&contextual_content)
            .await?;

        // Create relationship fingerprint
        let fingerprint = smart_embedding_cache::RelationshipFingerprint {
            parent_id: node.parent_id.clone(),
            sibling_ids: if let Some(parent_id) = &node.parent_id {
                self.get_children(parent_id)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|sibling| sibling.id != node.id)
                    .map(|sibling| sibling.id)
                    .collect()
            } else {
                Vec::new()
            },
            mention_ids: Vec::new(), // TODO: Extract mentions
            last_modified: Utc::now(),
        };

        // Cache the contextual embedding
        {
            let mut cache = self.embedding_cache.write().await;
            cache.cache_contextual_embedding(context_hash, embedding.clone(), fingerprint);
        }

        Ok(embedding)
    }

    /// Invalidate cache when node changes
    pub async fn invalidate_node_cache(&self, node_id: &NodeId) {
        let mut cache = self.embedding_cache.write().await;
        cache.invalidate_node_embeddings(node_id);
    }

    /// Build contextual content for embeddings
    async fn build_contextual_content(&self, node: &Node) -> NodeSpaceResult<String> {
        let mut contextual_parts = Vec::new();

        // Add the node's own content
        if let Some(content) = node.content.as_str() {
            contextual_parts.push(format!("Content: {}", content));
        }

        // Add parent context
        if let Some(parent_id) = &node.parent_id {
            if let Ok(Some(parent_node)) = self.data_store.get_node(parent_id).await {
                if let Some(parent_content) = parent_node.content.as_str() {
                    contextual_parts.push(format!("Parent: {}", parent_content));
                }
            }
        }

        // Add sibling context (limited to avoid overwhelming)
        if let Some(parent_id) = &node.parent_id {
            let siblings = self.get_children(parent_id).await.unwrap_or_default();
            let sibling_contents: Vec<String> = siblings
                .iter()
                .filter(|sibling| sibling.id != node.id)
                .take(3) // Limit to 3 siblings for context
                .filter_map(|sibling| sibling.content.as_str())
                .map(|content| format!("Sibling: {}", content))
                .collect();
            contextual_parts.extend(sibling_contents);
        }

        Ok(contextual_parts.join("\n"))
    }
}

impl NodeSpaceService<nodespace_data_store::LanceDataStore, nodespace_nlp_engine::LocalNLPEngine> {
    /// Factory method for service container dependency injection
    /// Creates a fully configured NodeSpace service with database and model paths
    pub async fn create_with_paths(
        database_path: &str,
        model_directory: Option<&str>,
    ) -> NodeSpaceResult<Self> {
        use nodespace_data_store::LanceDataStore;
        use nodespace_nlp_engine::LocalNLPEngine;

        // Initialize NLP engine with optional model directory
        let nlp_engine1 = if let Some(model_dir) = model_directory {
            LocalNLPEngine::with_model_directory(model_dir)
        } else {
            LocalNLPEngine::new() // Uses smart path resolution
        };

        // Create a second NLP engine instance for the adapter
        let nlp_engine2 = if let Some(model_dir) = model_directory {
            LocalNLPEngine::with_model_directory(model_dir)
        } else {
            LocalNLPEngine::new() // Uses smart path resolution
        };

        // Initialize data store with injected database path
        let mut data_store = LanceDataStore::new(database_path).await.map_err(|e| {
            NodeSpaceError::InternalError {
                message: format!(
                    "Failed to initialize data store at '{}': {}",
                    database_path, e
                ),
                service: "core-logic".to_string(),
            }
        })?;

        // Create an adapter to bridge NLP engine to data store's embedding generator interface
        let embedding_adapter = NLPEmbeddingAdapter::new(nlp_engine2);

        // Set the adapter as the embedding generator for automatic embedding handling
        data_store.set_embedding_generator(Box::new(embedding_adapter));

        Ok(Self::new(data_store, nlp_engine1))
    }

    /// Factory method for development environment
    /// Uses default development paths
    pub async fn create_for_development() -> NodeSpaceResult<Self> {
        Self::create_with_paths("../data/lance_db/development.db", Some("../models")).await
    }

    /// Factory method for testing environment
    /// Uses in-memory database and test model paths
    pub async fn create_for_testing() -> NodeSpaceResult<Self> {
        Self::create_with_paths(
            "memory", None, // Use smart path resolution for tests
        )
        .await
    }

    /// Factory method for production environment
    /// Uses environment variables or explicit paths
    pub async fn create_for_production(
        database_path: &str,
        model_directory: &str,
    ) -> NodeSpaceResult<Self> {
        Self::create_with_paths(database_path, Some(model_directory)).await
    }
}

impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> NodeSpaceService<D, N> {
    /// Initialize the service and load models
    pub async fn initialize(&self) -> NodeSpaceResult<()> {
        // Update state to initializing
        {
            let mut state = self.state.write().await;
            *state = ServiceState::Initializing;
        }

        // Initialize NLP engine with configuration
        match self.initialize_nlp_engine().await {
            Ok(_) => {
                let mut state = self.state.write().await;
                *state = ServiceState::Ready;
                Ok(())
            }
            Err(e) => {
                let mut state = self.state.write().await;
                *state = ServiceState::Failed(format!("NLP engine initialization failed: {}", e));
                Err(e)
            }
        }
    }

    /// Get current service state
    pub async fn get_state(&self) -> ServiceState {
        self.state.read().await.clone()
    }

    /// Check if service is ready for operations
    pub async fn is_ready(&self) -> bool {
        matches!(self.get_state().await, ServiceState::Ready)
    }

    /// Internal method to initialize NLP engine
    async fn initialize_nlp_engine(&self) -> NodeSpaceResult<()> {
        // For now, we'll validate the NLP engine by attempting a simple operation
        // This ensures the models are loaded and ready
        match self
            .nlp_engine
            .generate_embedding("initialization test")
            .await
        {
            Ok(_) => {
                // Test text generation as well
                match self.nlp_engine.generate_text("test initialization").await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if self.config.offline_config.enable_offline {
                            // Log warning but continue in offline mode
                            eprintln!("Warning: Text generation initialization failed, continuing in offline mode: {}", e);
                            Ok(())
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            Err(e) => {
                if self.config.offline_config.enable_offline {
                    // Log warning but continue in offline mode
                    eprintln!("Warning: Embedding generation initialization failed, continuing in offline mode: {}", e);
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Graceful shutdown of the service
    pub async fn shutdown(&self) -> NodeSpaceResult<()> {
        let mut state = self.state.write().await;
        *state = ServiceState::Uninitialized;
        Ok(())
    }
}

/// Core business logic operations interface following distributed contract pattern
#[async_trait]
pub trait CoreLogic: Send + Sync {
    /// Create a new knowledge node with AI processing
    async fn create_knowledge_node(
        &self,
        content: &str,
        metadata: serde_json::Value,
    ) -> NodeSpaceResult<NodeId>;

    /// Create a node for a specific date with automatic parent relationship
    async fn create_node_for_date(
        &self,
        date: NaiveDate,
        content: &str,
        node_type: NodeType,
        metadata: Option<serde_json::Value>,
    ) -> NodeSpaceResult<NodeId>;

    /// Get all text nodes for a specific date
    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>>;

    /// Navigate to a specific date with navigation context
    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult>;

    /// Find an existing date node by date (schema-based indexed lookup)
    async fn find_date_node(&self, date: NaiveDate) -> NodeSpaceResult<Option<NodeId>>;

    /// Ensure a date node exists, creating it if necessary (atomic find-or-create)
    async fn ensure_date_node_exists(&self, date: NaiveDate) -> NodeSpaceResult<NodeId>;

    /// Get date structure with hierarchical children for a specific date
    async fn get_nodes_for_date_with_structure(&self, date: NaiveDate) -> NodeSpaceResult<DateStructure>;

    /// Search for nodes using semantic similarity
    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    /// Process natural language query and return results
    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse>;

    /// Update node content and reprocess embeddings
    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()>;

    /// Get related nodes using graph relationships
    async fn get_related_nodes(
        &self,
        node_id: &NodeId,
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<Vec<NodeId>>;

    /// Generate insights from a collection of nodes
    async fn generate_insights(&self, node_ids: Vec<NodeId>) -> NodeSpaceResult<String>;

    /// Update node hierarchical relationships and sibling ordering
    async fn update_node_structure(
        &self,
        node_id: &NodeId,
        operation: &str,
        target_parent_id: Option<&NodeId>,
        previous_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()>;

    /// Set node's parent (for indent/outdent operations)
    async fn set_node_parent(
        &self,
        node_id: &NodeId,
        parent_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()>;

    /// Update sibling order (for move up/down operations)
    async fn update_sibling_order(
        &self,
        node_id: &NodeId,
        previous_sibling_id: Option<&NodeId>,
        next_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()>;

    /// Batch lookup for multiple node relationships (optimization for N+1 queries)
    async fn get_batch_related_nodes(
        &self,
        node_ids: &[NodeId],
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<std::collections::HashMap<NodeId, Vec<NodeId>>>;
}

/// Cross-modal search orchestration interface for complex queries
#[async_trait]
pub trait CrossModalSearch: Send + Sync {
    /// Intelligent cross-modal search with entity and temporal extraction
    async fn intelligent_cross_modal_search(
        &self,
        query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    /// Extract entities from natural language queries
    async fn extract_entities(&self, query: &str) -> NodeSpaceResult<ExtractedEntities>;

    /// Extract temporal references from queries
    async fn extract_temporal_refs(&self, query: &str) -> NodeSpaceResult<Vec<TemporalReference>>;

    /// Extract visual attributes from queries
    async fn extract_visual_refs(&self, query: &str) -> NodeSpaceResult<VisualAttributes>;

    /// Multi-strategy search coordination
    async fn multi_strategy_search(
        &self,
        query_embedding: Vec<f32>,
        entities: &ExtractedEntities,
        temporal_refs: &[TemporalReference],
        visual_refs: &VisualAttributes,
    ) -> NodeSpaceResult<Vec<SearchResult>>;

    /// Intelligent result fusion and ranking
    async fn intelligent_result_fusion(
        &self,
        search_results: Vec<SearchResult>,
        original_query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>>;
}


/// Hierarchy computation operations for runtime node relationships
#[async_trait]
pub trait HierarchyComputation: Send + Sync {
    /// Compute node depth by traversing parent chain
    async fn get_node_depth(&self, node_id: &NodeId) -> NodeSpaceResult<u32>;

    /// Get direct children of a parent node
    async fn get_children(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Get all ancestors of a node (parent, grandparent, etc.)
    async fn get_ancestors(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Get all siblings of a node (same parent)
    async fn get_siblings(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Move a node to a new parent (validates hierarchy constraints)
    async fn move_node(&self, node_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()>;

    /// Move an entire subtree to a new parent
    async fn move_subtree(&self, root_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()>;

    /// Get a subtree with computed depths for each node
    async fn get_subtree_with_depths(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<(Node, u32)>>;

    /// Validate that a hierarchy move operation is legal (no cycles, valid targets)
    async fn validate_hierarchy_move(
        &self,
        node_id: &NodeId,
        new_parent: &NodeId,
    ) -> NodeSpaceResult<()>;

    /// Invalidate hierarchy cache (call after structural changes)
    async fn invalidate_hierarchy_cache(&self);

    /// Get all nodes in a tree/subtree rooted at the given node
    async fn get_tree_nodes(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<Node>>;

    /// Check if one node is an ancestor of another
    async fn is_ancestor_of(
        &self,
        potential_ancestor: &NodeId,
        node_id: &NodeId,
    ) -> NodeSpaceResult<bool>;
}

/// Search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub node_id: NodeId,
    pub node: Node,
    pub score: f32,
}

/// Cross-modal search entities extracted from queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub people: Vec<String>,
    pub events: Vec<String>,
    pub objects: Vec<String>,
    pub locations: Vec<String>,
}

/// Temporal references extracted from queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalReference {
    pub raw_text: String,
    pub parsed_date: Option<NaiveDate>,
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    pub temporal_type: TemporalType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalType {
    Exact,    // "on June 15"
    Relative, // "yesterday", "last week"
    Event,    // "during Claire's birthday"
    Fuzzy,    // "around that time"
}

/// Visual attributes extracted from queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAttributes {
    pub colors: Vec<String>,
    pub objects: Vec<String>,
    pub scene_types: Vec<String>,
    pub people_descriptions: Vec<String>,
}

/// Multi-strategy search configuration for intelligent fusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStrategyConfig {
    pub semantic_weight: f32,
    pub entity_weight: f32,
    pub temporal_weight: f32,
    pub visual_weight: f32,
    pub max_results_per_strategy: usize,
    pub final_result_limit: usize,
}

/// Query response with results and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub answer: String,
    pub sources: Vec<NodeId>,
    pub confidence: f32,
    pub related_queries: Vec<String>,
}

/// Navigation result for date-based navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub date: NaiveDate,
    pub nodes: Vec<Node>,
    pub has_previous: bool,
    pub has_next: bool,
}

/// Structured date representation with hierarchical organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateStructure {
    pub date_node: Node,
    pub children: Vec<OrderedNode>,
    pub has_content: bool,
}

/// Hierarchically ordered node with position metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderedNode {
    pub node: Node,
    pub children: Vec<OrderedNode>,
    pub depth: u32,
    pub sibling_index: u32,
}

#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> CoreLogic for NodeSpaceService<D, N> {
    async fn create_knowledge_node(
        &self,
        content: &str,
        metadata: serde_json::Value,
    ) -> NodeSpaceResult<NodeId> {
        let timer = self
            .performance_monitor
            .start_operation("create_knowledge_node")
            .with_metadata("content_length".to_string(), content.len().to_string());

        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            let error = NodeSpaceError::InternalError {
                message: format!("Service not ready: {:?}", state),
                service: "core-logic".to_string(),
            };
            timer.complete_error(error.to_string());
            return Err(error);
        }

        // Create the node with provided content and metadata
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();

        let node = Node {
            id: node_id.clone(),
            content: serde_json::Value::String(content.to_string()),
            metadata: Some(metadata),
            created_at: now.clone(),
            updated_at: now,
            parent_id: None,
            next_sibling: None,
            previous_sibling: None,
        };

        // Store the node - data store will automatically generate embeddings using the EmbeddingGenerator
        self.data_store.store_node(node).await?;

        timer.complete_success();
        Ok(node_id)
    }

    async fn create_node_for_date(
        &self,
        date: NaiveDate,
        content: &str,
        _node_type: NodeType,
        metadata: Option<serde_json::Value>,
    ) -> NodeSpaceResult<NodeId> {
        let timer = self
            .performance_monitor
            .start_operation("create_node_for_date")
            .with_metadata("date".to_string(), date.to_string())
            .with_metadata("content_length".to_string(), content.len().to_string());

        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            let error = NodeSpaceError::InternalError {
                message: format!("Service not ready: {:?}", state),
                service: "core-logic".to_string(),
            };
            timer.complete_error(error.to_string());
            return Err(error);
        }

        // Ensure date node exists (creates if necessary using NodeType::Date schema)
        let date_node_id = self.ensure_date_node_exists(date).await?;

        // Create new node with proper parent relationship
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();

        let mut node = Node {
            id: node_id.clone(),
            content: serde_json::Value::String(content.to_string()),
            metadata,
            created_at: now.clone(),
            updated_at: now,
            parent_id: Some(date_node_id.clone()),
            next_sibling: None,
            previous_sibling: None,
        };

        // Handle sibling ordering (add as last child)
        let siblings = self.get_children(&date_node_id).await?;
        if let Some(last_sibling) = siblings.last() {
            // Update sibling pointers
            node.previous_sibling = Some(last_sibling.id.clone());
            // Note: We'll need to update the last sibling's next_sibling pointer
            // This would require updating the existing node in the data store
        }

        // Store node with embedding generation
        self.data_store.store_node(node).await?;

        // Update the previous sibling's next_sibling pointer if there was one
        if let Some(previous_sibling_id) = &siblings.last().map(|s| &s.id) {
            if let Ok(Some(mut prev_sibling)) = self.data_store.get_node(previous_sibling_id).await {
                prev_sibling.next_sibling = Some(node_id.clone());
                self.data_store.store_node(prev_sibling).await?;
            }
        }

        timer.complete_success();
        Ok(node_id)
    }

    async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError {
                message: format!("Service not ready: {:?}", state),
                service: "core-logic".to_string(),
            });
        }

        // Apply performance configuration limits
        let effective_limit = if let Some(max_batch) = self.config.performance_config.max_batch_size
        {
            limit.min(max_batch)
        } else {
            limit
        };

        // For LanceDB: Simple content search using query_nodes
        let all_nodes = self.data_store.query_nodes(query).await?;
        let nodes: Vec<_> = all_nodes.into_iter().take(effective_limit).collect();

        // Convert to SearchResult with basic scoring
        let mut results = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
            // Simple scoring based on position (higher position = lower score)
            let score = 1.0 - (index as f32 * constants::SCORE_DECAY_FACTOR);

            results.push(SearchResult {
                node_id: node.id.clone(),
                node,
                score: score.max(constants::MIN_SEARCH_SCORE),
            });
        }

        Ok(results)
    }

    async fn process_query(&self, query: &str) -> NodeSpaceResult<QueryResponse> {
        let timer = self
            .performance_monitor
            .start_operation("process_query")
            .with_metadata("query_length".to_string(), query.len().to_string());

        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            let error = NodeSpaceError::InternalError {
                message: format!("Service not ready: {:?}", state),
                service: "core-logic".to_string(),
            };
            timer.complete_error(error.to_string());
            return Err(error);
        }

        // Step 1: Gather context from semantic search
        let (context, sources) = self.gather_query_context(query).await?;

        // Step 2: Build and execute prompt
        let prompt = self.build_contextual_prompt(query, &context);
        let answer = self.generate_contextual_answer(&prompt, &sources).await?;

        // Step 3: Calculate confidence and generate suggestions
        let confidence = self.calculate_response_confidence(&context, &answer);
        let related_queries = self.generate_related_queries(query);

        let response = QueryResponse {
            answer,
            sources,
            confidence,
            related_queries,
        };

        timer.complete_success();
        Ok(response)
    }

    async fn get_batch_related_nodes(
        &self,
        node_ids: &[NodeId],
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<std::collections::HashMap<NodeId, Vec<NodeId>>> {
        // Use the implementation from the impl block
        NodeSpaceService::get_batch_related_nodes(self, node_ids, relationship_types).await
    }

    async fn update_node(&self, node_id: &NodeId, content: &str) -> NodeSpaceResult<()> {
        // Get existing node
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "node".to_string(), 
                id: node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        // Update content and timestamp
        node.content = serde_json::Value::String(content.to_string());
        node.updated_at = chrono::Utc::now().to_rfc3339();

        // Use the data store's update method which handles embedding regeneration automatically
        // The LanceDB data store now detects content changes and regenerates embeddings as needed
        self.data_store.update_node(node).await?;

        // Invalidate embedding cache for this node and its dependents
        self.invalidate_node_cache(node_id).await;

        Ok(())
    }

    async fn get_related_nodes(
        &self,
        node_id: &NodeId,
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<Vec<NodeId>> {
        // For MVP, use a simple query to find related nodes
        let _relationship_filters = if relationship_types.is_empty() {
            "".to_string()
        } else {
            format!(
                " AND type IN [{}]",
                relationship_types
                    .iter()
                    .map(|t| format!("'{}'", t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        // Optimized relationship lookup using batch operations
        // Instead of loading all nodes, we can use targeted queries
        let related_nodes = if relationship_types.is_empty() {
            // Search for any mentions of the target node ID
            self.data_store
                .query_nodes(&node_id.to_string())
                .await
                .unwrap_or_default()
        } else {
            // For specific relationship types, we'd need more sophisticated querying
            // For now, fall back to the general approach but cache results
            let all_nodes = self.data_store.query_nodes("").await.unwrap_or_default();
            all_nodes
                .into_iter()
                .filter(|node| {
                    // Check if this node has a relationship to our target node
                    if let Some(metadata) = &node.metadata {
                        if let Some(mentions) = metadata.get("mentions") {
                            if let Some(mentions_array) = mentions.as_array() {
                                return mentions_array
                                    .iter()
                                    .any(|mention| mention.as_str() == Some(node_id.as_str()));
                            }
                        }
                        // Also check for specific relationship types in metadata
                        for rel_type in &relationship_types {
                            if let Some(relationships) = metadata.get("relationships") {
                                if let Some(rel_obj) = relationships.as_object() {
                                    if let Some(targets) = rel_obj.get(rel_type) {
                                        if let Some(targets_array) = targets.as_array() {
                                            if targets_array
                                                .iter()
                                                .any(|t| t.as_str() == Some(node_id.as_str()))
                                            {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    false
                })
                .collect()
        };

        let related_ids: Vec<NodeId> = related_nodes.into_iter().map(|node| node.id).collect();

        Ok(related_ids)
    }

    async fn generate_insights(&self, node_ids: Vec<NodeId>) -> NodeSpaceResult<String> {
        if node_ids.is_empty() {
            return Ok("No nodes provided for insight generation.".to_string());
        }

        // Collect content from all specified nodes
        let mut contents = Vec::new();
        for node_id in &node_ids {
            if let Ok(Some(node)) = self.data_store.get_node(node_id).await {
                if let Some(content_str) = node.content.as_str() {
                    contents.push(content_str.to_string());
                }
            }
        }

        if contents.is_empty() {
            return Ok("No readable content found in the specified nodes.".to_string());
        }

        // Generate insights using LLM
        let combined_content = contents.join("\n\n---\n\n");
        let prompt = format!(
            "Analyze the following content and provide key insights, patterns, and connections:\n\n{}\n\nProvide a concise summary with 3-5 key insights:",
            combined_content
        );

        let insights = self.nlp_engine.generate_text(&prompt).await?;

        Ok(insights)
    }

    /// Update node hierarchical relationships and sibling ordering
    async fn update_node_structure(
        &self,
        node_id: &NodeId,
        operation: &str,
        target_parent_id: Option<&NodeId>,
        previous_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()> {
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "node".to_string(), 
                id: node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        match operation {
            "indent" => {
                // Make node a child of target parent
                if let Some(parent_id) = target_parent_id {
                    let mut metadata = node.metadata.unwrap_or_else(|| serde_json::json!({}));
                    metadata["parent_id"] = serde_json::Value::String(parent_id.to_string());
                    node.metadata = Some(metadata);

                    // Update sibling relationships if provided
                    node.previous_sibling = previous_sibling_id.cloned();
                    node.next_sibling = None; // Will be updated by subsequent operations if needed
                }
            }
            "outdent" => {
                // Remove parent relationship
                if let Some(metadata) = &mut node.metadata {
                    metadata
                        .as_object_mut()
                        .and_then(|obj| obj.remove("parent_id"));
                }

                // Reset sibling relationships
                node.previous_sibling = previous_sibling_id.cloned();
                node.next_sibling = None;
            }
            "move_up" | "move_down" => {
                // Update sibling order without changing parent
                node.previous_sibling = previous_sibling_id.cloned();
                // next_sibling will be handled by update_sibling_order if needed
            }
            "reorder" => {
                // Move to specific position in sibling list
                node.previous_sibling = previous_sibling_id.cloned();
            }
            _ => {
                return Err(NodeSpaceError::Validation(ValidationError::InvalidFormat { 
                    field: "operation".to_string(), 
                    expected: "indent|outdent|move_up|move_down|reorder".to_string(), 
                    actual: operation.to_string(), 
                    examples: vec!["indent".to_string(), "outdent".to_string()] 
                }))
            }
        }

        // Update the node using the data store's update method
        self.data_store.update_node(node).await?;
        Ok(())
    }

    /// Set node's parent (for indent/outdent operations)
    async fn set_node_parent(
        &self,
        node_id: &NodeId,
        parent_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()> {
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "node".to_string(), 
                id: node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        let mut metadata = node.metadata.unwrap_or_else(|| serde_json::json!({}));

        if let Some(parent_id) = parent_id {
            // Set parent relationship
            metadata["parent_id"] = serde_json::Value::String(parent_id.to_string());
        } else {
            // Remove parent relationship
            metadata
                .as_object_mut()
                .and_then(|obj| obj.remove("parent_id"));
        }

        node.metadata = Some(metadata);
        self.data_store.update_node(node).await?;
        Ok(())
    }

    /// Update sibling order (for move up/down operations)
    async fn update_sibling_order(
        &self,
        node_id: &NodeId,
        previous_sibling_id: Option<&NodeId>,
        next_sibling_id: Option<&NodeId>,
    ) -> NodeSpaceResult<()> {
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "node".to_string(), 
                id: node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        // Update the node's sibling pointers
        node.previous_sibling = previous_sibling_id.cloned();
        node.next_sibling = next_sibling_id.cloned();

        // Update the updated node using the data store's update method
        self.data_store.update_node(node).await?;

        // Update affected siblings' pointers to maintain consistency
        if let Some(prev_id) = previous_sibling_id {
            if let Some(mut prev_node) = self.data_store.get_node(prev_id).await? {
                prev_node.next_sibling = Some(node_id.clone());
                self.data_store.update_node(prev_node).await?;
            }
        }

        if let Some(next_id) = next_sibling_id {
            if let Some(mut next_node) = self.data_store.get_node(next_id).await? {
                next_node.previous_sibling = Some(node_id.clone());
                self.data_store.update_node(next_node).await?;
            }
        }

        Ok(())
    }

    async fn get_nodes_for_date(&self, date: NaiveDate) -> NodeSpaceResult<Vec<Node>> {
        let timer = self
            .performance_monitor
            .start_operation("get_nodes_for_date")
            .with_metadata("date".to_string(), date.to_string());

        // Find the date node first
        if let Some(date_node_id) = self.find_date_node(date).await? {
            // Get all children of the date node
            let nodes = self.get_children(&date_node_id).await?;
            timer.complete_success();
            Ok(nodes)
        } else {
            // No date node exists, return empty list
            timer.complete_success();
            Ok(vec![])
        }
    }

    async fn navigate_to_date(&self, date: NaiveDate) -> NodeSpaceResult<NavigationResult> {
        let timer = self
            .performance_monitor
            .start_operation("navigate_to_date")
            .with_metadata("date".to_string(), date.to_string());

        let nodes = self.get_nodes_for_date(date).await?;
        
        // Check for previous/next dates (simplified implementation)
        let has_previous = true; // TODO: Implement actual previous date checking
        let has_next = true; // TODO: Implement actual next date checking

        let result = NavigationResult {
            date,
            nodes,
            has_previous,
            has_next,
        };

        timer.complete_success();
        Ok(result)
    }

    async fn find_date_node(&self, date: NaiveDate) -> NodeSpaceResult<Option<NodeId>> {
        let timer = self
            .performance_monitor
            .start_operation("find_date_node")
            .with_metadata("date".to_string(), date.to_string());

        // Use date metadata to find the date node efficiently
        let date_str = date.format("%Y-%m-%d").to_string();
        
        // For now, search through all nodes to find one with date metadata
        // In a real implementation, this would use indexed lookup
        let all_nodes = self.data_store.query_nodes("").await?;
        
        for node in all_nodes {
            if let Some(metadata) = &node.metadata {
                if let Some(node_date) = metadata.get("date") {
                    if node_date.as_str() == Some(&date_str) {
                        timer.complete_success();
                        return Ok(Some(node.id));
                    }
                }
            }
        }

        timer.complete_success();
        Ok(None)
    }

    async fn ensure_date_node_exists(&self, date: NaiveDate) -> NodeSpaceResult<NodeId> {
        let timer = self
            .performance_monitor
            .start_operation("ensure_date_node_exists")
            .with_metadata("date".to_string(), date.to_string());

        // Check if date node already exists
        if let Some(existing_id) = self.find_date_node(date).await? {
            timer.complete_success();
            return Ok(existing_id);
        }

        // Create new date node with proper format
        let date_content = format!("# {}", date.format("%B %d, %Y"));
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();

        let mut metadata = serde_json::Map::new();
        metadata.insert("date".to_string(), serde_json::Value::String(date.format("%Y-%m-%d").to_string()));
        metadata.insert("node_type".to_string(), serde_json::Value::String("date".to_string()));

        let date_node = Node {
            id: node_id.clone(),
            content: serde_json::Value::String(date_content),
            metadata: Some(serde_json::Value::Object(metadata)),
            created_at: now.clone(),
            updated_at: now,
            parent_id: None, // Date nodes are top-level
            next_sibling: None,
            previous_sibling: None,
        };

        self.data_store.store_node(date_node).await?;

        timer.complete_success();
        Ok(node_id)
    }

    async fn get_nodes_for_date_with_structure(&self, date: NaiveDate) -> NodeSpaceResult<DateStructure> {
        let timer = self
            .performance_monitor
            .start_operation("get_nodes_for_date_with_structure")
            .with_metadata("date".to_string(), date.to_string());

        // Ensure date node exists first
        let date_node_id = self.ensure_date_node_exists(date).await?;

        // Get the date node
        let date_node = self.data_store.get_node(&date_node_id).await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "date_node".to_string(), 
                id: date_node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        // Get hierarchical structure
        let children = self.build_ordered_hierarchy(&date_node_id, 1).await?;
        let has_content = !children.is_empty();

        let structure = DateStructure {
            date_node,
            children,
            has_content,
        };

        timer.complete_success();
        Ok(structure)
    }
}

/// Legacy CoreLogic interface for backward compatibility
#[async_trait]
pub trait LegacyCoreLogic {
    /// Create a new node with automatic embedding generation
    async fn create_node(
        &self,
        content: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> NodeSpaceResult<NodeId>;

    /// Retrieve a node by ID
    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>>;

    /// Delete a node
    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()>;

    /// Search nodes using semantic and text search
    async fn search_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>>;

    /// Process a RAG query: search for context + generate response
    async fn process_rag_query(&self, query: &str) -> NodeSpaceResult<String>;

    /// Create a relationship between nodes
    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()>;
}

/// Legacy implementation for backward compatibility
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> LegacyCoreLogic
    for NodeSpaceService<D, N>
{
    async fn create_node(
        &self,
        content: serde_json::Value,
        metadata: Option<serde_json::Value>,
    ) -> NodeSpaceResult<NodeId> {
        let node_id = NodeId::new();
        let now = chrono::Utc::now().to_rfc3339();

        let node = Node {
            id: node_id.clone(),
            content,
            metadata,
            created_at: now.clone(),
            updated_at: now,
            parent_id: None,
            next_sibling: None,
            previous_sibling: None,
        };

        self.data_store.store_node(node).await?;
        Ok(node_id)
    }

    async fn get_node(&self, id: &NodeId) -> NodeSpaceResult<Option<Node>> {
        self.data_store.get_node(id).await
    }

    async fn delete_node(&self, id: &NodeId) -> NodeSpaceResult<()> {
        self.data_store.delete_node(id).await
    }

    async fn search_nodes(&self, query: &str) -> NodeSpaceResult<Vec<Node>> {
        // For LanceDB: Simple content search
        self.data_store.query_nodes(query).await
    }

    async fn process_rag_query(&self, query: &str) -> NodeSpaceResult<String> {
        let response = self.process_query(query).await?;
        Ok(response.answer)
    }

    async fn create_relationship(
        &self,
        from: &NodeId,
        to: &NodeId,
        rel_type: &str,
    ) -> NodeSpaceResult<()> {
        self.data_store
            .create_relationship(from, to, rel_type)
            .await
    }
}


/// Hierarchy computation implementation for NodeSpaceService
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> HierarchyComputation
    for NodeSpaceService<D, N>
{
    async fn get_node_depth(&self, node_id: &NodeId) -> NodeSpaceResult<u32> {
        // Check cache first
        {
            let cache = self.hierarchy_cache.read().await;
            if let Some(depth) = cache.get_depth(node_id) {
                return Ok(depth);
            }
        }

        // Compute depth by traversing parent chain
        let mut depth = 0;
        let mut current_node_id = node_id.clone();

        loop {
            // Get the current node
            let node = self
                .data_store
                .get_node(&current_node_id)
                .await?
                .ok_or_else(|| {
                    NodeSpaceError::Database(DatabaseError::NotFound { 
                        entity_type: "node".to_string(), 
                        id: current_node_id.to_string(), 
                        suggestions: vec![] 
                    })
                })?;

            // Check if this node has a parent
            if let Some(parent_id) = &node.parent_id {
                depth += 1;
                current_node_id = parent_id.clone();

                // Safety check to prevent infinite loops
                if depth > 1000 {
                    return Err(NodeSpaceError::Validation(ValidationError::InvalidFormat { 
                        field: "hierarchy_depth".to_string(), 
                        expected: "<1000".to_string(), 
                        actual: "exceeds_limit".to_string(), 
                        examples: vec!["100".to_string(), "500".to_string()] 
                    }));
                }
            } else {
                // Reached root node
                break;
            }
        }

        // Cache the result
        {
            let mut cache = self.hierarchy_cache.write().await;
            cache.cache_depth(node_id.clone(), depth);
        }

        Ok(depth)
    }

    async fn get_children(&self, parent_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Validate that the parent node exists
        self.data_store.get_node(parent_id).await?.ok_or_else(|| {
            NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "parent node".to_string(), 
                id: parent_id.to_string(), 
                suggestions: vec![] 
            })
        })?;

        // Check cache first
        {
            let cache = self.hierarchy_cache.read().await;
            if let Some(cached_child_ids) = cache.get_children(parent_id) {
                // Load nodes from cached IDs
                let mut children = Vec::new();
                for child_id in cached_child_ids {
                    if let Ok(Some(child)) = self.data_store.get_node(child_id).await {
                        children.push(child);
                    }
                }
                return Ok(children);
            }
        }

        // Get all nodes and filter for children
        let all_nodes = self.data_store.query_nodes("").await?;
        let mut children = Vec::new();
        let mut child_ids = Vec::new();

        for node in all_nodes {
            if let Some(node_parent_id) = &node.parent_id {
                if *node_parent_id == *parent_id {
                    child_ids.push(node.id.clone());
                    children.push(node);
                }
            }
        }

        // Cache the child IDs
        {
            let mut cache = self.hierarchy_cache.write().await;
            cache.cache_children(parent_id.clone(), child_ids);
        }

        Ok(children)
    }

    async fn get_ancestors(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        let mut ancestors = Vec::new();
        let mut current_node_id = node_id.clone();

        loop {
            // Get the current node
            let node = self
                .data_store
                .get_node(&current_node_id)
                .await?
                .ok_or_else(|| {
                    NodeSpaceError::Database(DatabaseError::NotFound { 
                        entity_type: "node".to_string(), 
                        id: current_node_id.to_string(), 
                        suggestions: vec![] 
                    })
                })?;

            // Check if this node has a parent
            if let Some(parent_id) = &node.parent_id {
                // Get the parent node
                let parent_node = self.data_store.get_node(parent_id).await?.ok_or_else(|| {
                    NodeSpaceError::Database(DatabaseError::NotFound { 
                        entity_type: "parent node".to_string(), 
                        id: parent_id.to_string(), 
                        suggestions: vec![] 
                    })
                })?;

                ancestors.push(parent_node);
                current_node_id = parent_id.clone();

                // Safety check to prevent infinite loops
                if ancestors.len() > 1000 {
                    return Err(NodeSpaceError::Validation(ValidationError::InvalidFormat { 
                        field: "ancestry_chain".to_string(), 
                        expected: "<1000".to_string(), 
                        actual: "exceeds_limit".to_string(), 
                        examples: vec!["100".to_string(), "500".to_string()] 
                    }));
                }
            } else {
                // Reached root node
                break;
            }
        }

        Ok(ancestors)
    }

    async fn get_siblings(&self, node_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        // Get the node to find its parent
        let node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "node".to_string(), 
                id: node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        // If node has no parent, it has no siblings
        let parent_id = match &node.parent_id {
            Some(parent_id) => parent_id.clone(),
            None => return Ok(Vec::new()),
        };

        // Get all children of the parent, excluding the original node
        let all_children = self.get_children(&parent_id).await?;
        let siblings: Vec<Node> = all_children
            .into_iter()
            .filter(|child| child.id != *node_id)
            .collect();

        Ok(siblings)
    }

    async fn move_node(&self, node_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()> {
        // Validate the move is legal
        self.validate_hierarchy_move(node_id, new_parent).await?;

        // Get the node to move
        let mut node = self
            .data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "node".to_string(), 
                id: node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        // Update the node's parent_id directly
        node.parent_id = Some(new_parent.clone());
        node.updated_at = chrono::Utc::now().to_rfc3339();

        // Save the updated node
        self.data_store.update_node(node).await?;

        // Invalidate cache since hierarchy changed
        self.invalidate_hierarchy_cache().await;

        Ok(())
    }

    async fn move_subtree(&self, root_id: &NodeId, new_parent: &NodeId) -> NodeSpaceResult<()> {
        // Validate the move is legal for the root
        self.validate_hierarchy_move(root_id, new_parent).await?;

        // Get all descendants of the subtree
        let all_nodes = self.data_store.query_nodes("").await?;
        let descendants = get_all_descendants(&all_nodes, root_id);

        // Validate each descendant won't create cycles
        for descendant in &descendants {
            if descendant.id == *new_parent {
                return Err(NodeSpaceError::Validation(ValidationError::InvalidFormat { 
                        field: "subtree_move".to_string(), 
                        expected: "no_cycle".to_string(), 
                        actual: "creates_cycle".to_string(), 
                        examples: vec!["valid_parent".to_string()] 
                }));
            }
        }

        // Move the root node
        self.move_node(root_id, new_parent).await?;

        Ok(())
    }

    async fn get_subtree_with_depths(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<(Node, u32)>> {
        let mut result = Vec::new();

        // Get the root node and its depth
        let root_node = self.data_store.get_node(root_id).await?.ok_or_else(|| {
            NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "root node".to_string(), 
                id: root_id.to_string(), 
                suggestions: vec![] 
            })
        })?;
        let root_depth = self.get_node_depth(root_id).await?;
        result.push((root_node, root_depth));

        // Get all descendants
        let all_nodes = self.data_store.query_nodes("").await?;
        let descendants = get_all_descendants(&all_nodes, root_id);

        // Compute depth for each descendant
        for descendant in descendants {
            let depth = self.get_node_depth(&descendant.id).await?;
            result.push((descendant, depth));
        }

        // Sort by depth for consistent ordering
        result.sort_by(|a, b| a.1.cmp(&b.1));

        Ok(result)
    }

    async fn validate_hierarchy_move(
        &self,
        node_id: &NodeId,
        new_parent: &NodeId,
    ) -> NodeSpaceResult<()> {
        // Check if node exists
        self.data_store
            .get_node(node_id)
            .await?
            .ok_or_else(|| NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "node".to_string(), 
                id: node_id.to_string(), 
                suggestions: vec![] 
            }))?;

        // Check if new parent exists
        self.data_store.get_node(new_parent).await?.ok_or_else(|| {
            NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "new parent".to_string(), 
                id: new_parent.to_string(), 
                suggestions: vec![] 
            })
        })?;

        // Check if moving to self (invalid)
        if node_id == new_parent {
            return Err(NodeSpaceError::Validation(ValidationError::InvalidFormat { 
                        field: "move_target".to_string(), 
                        expected: "different_node".to_string(), 
                        actual: "self".to_string(), 
                        examples: vec!["other_node_id".to_string()] 
            }));
        }

        // Check if new parent is a descendant of the node (would create cycle)
        let descendants = {
            let all_nodes = self.data_store.query_nodes("").await?;
            get_all_descendants(&all_nodes, node_id)
        };

        for descendant in descendants {
            if descendant.id == *new_parent {
                return Err(NodeSpaceError::Validation(ValidationError::InvalidFormat { 
                        field: "move_parent".to_string(), 
                        expected: "non_descendant".to_string(), 
                        actual: "descendant".to_string(), 
                        examples: vec!["sibling_node".to_string(), "parent_node".to_string()] 
                }));
            }
        }

        Ok(())
    }

    async fn invalidate_hierarchy_cache(&self) {
        let mut cache = self.hierarchy_cache.write().await;
        cache.invalidate();
    }

    async fn get_tree_nodes(&self, root_id: &NodeId) -> NodeSpaceResult<Vec<Node>> {
        let mut result = Vec::new();

        // Get the root node
        let root_node = self.data_store.get_node(root_id).await?.ok_or_else(|| {
            NodeSpaceError::Database(DatabaseError::NotFound { 
                entity_type: "root node".to_string(), 
                id: root_id.to_string(), 
                suggestions: vec![] 
            })
        })?;
        result.push(root_node);

        // Get all descendants
        let all_nodes = self.data_store.query_nodes("").await?;
        let descendants = get_all_descendants(&all_nodes, root_id);
        result.extend(descendants);

        Ok(result)
    }

    async fn is_ancestor_of(
        &self,
        potential_ancestor: &NodeId,
        node_id: &NodeId,
    ) -> NodeSpaceResult<bool> {
        // Get all ancestors of the node
        let ancestors = self.get_ancestors(node_id).await?;

        // Check if the potential ancestor is in the list
        Ok(ancestors
            .iter()
            .any(|ancestor| ancestor.id == *potential_ancestor))
    }
}

/// Helper function to get all descendants (children, grandchildren, etc.) of a node
fn get_all_descendants(all_nodes: &[Node], parent_id: &NodeId) -> Vec<Node> {
    let mut descendants = Vec::new();
    let mut to_process = vec![parent_id.clone()];

    while let Some(current_parent_id) = to_process.pop() {
        // Find direct children of current parent
        for node in all_nodes {
            if let Some(node_parent_id) = &node.parent_id {
                if *node_parent_id == current_parent_id {
                    // This is a child - add it to results and queue for processing
                    descendants.push(node.clone());
                    to_process.push(node.id.clone());
                }
            }
        }
    }

    descendants
}

/// Smart caching for hierarchy operations
#[derive(Debug, Default)]
pub struct HierarchyCache {
    depth_cache: std::collections::HashMap<NodeId, u32>,
    children_cache: std::collections::HashMap<NodeId, Vec<NodeId>>,
    last_updated: Option<std::time::Instant>,
    cache_ttl: std::time::Duration,
}

impl HierarchyCache {
    pub fn new() -> Self {
        Self {
            depth_cache: std::collections::HashMap::new(),
            children_cache: std::collections::HashMap::new(),
            last_updated: None,
            cache_ttl: std::time::Duration::from_secs(300), // 5 minute TTL
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(last_updated) = self.last_updated {
            last_updated.elapsed() > self.cache_ttl
        } else {
            true
        }
    }

    pub fn invalidate(&mut self) {
        self.depth_cache.clear();
        self.children_cache.clear();
        self.last_updated = None;
    }

    pub fn get_depth(&self, node_id: &NodeId) -> Option<u32> {
        if self.is_expired() {
            None
        } else {
            self.depth_cache.get(node_id).copied()
        }
    }

    pub fn cache_depth(&mut self, node_id: NodeId, depth: u32) {
        self.depth_cache.insert(node_id, depth);
        self.last_updated = Some(std::time::Instant::now());
    }

    pub fn get_children(&self, parent_id: &NodeId) -> Option<&Vec<NodeId>> {
        if self.is_expired() {
            None
        } else {
            self.children_cache.get(parent_id)
        }
    }

    pub fn cache_children(&mut self, parent_id: NodeId, children: Vec<NodeId>) {
        self.children_cache.insert(parent_id, children);
        self.last_updated = Some(std::time::Instant::now());
    }
}

/// Cross-modal search orchestration implementation for NodeSpaceService
#[async_trait]
impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> CrossModalSearch
    for NodeSpaceService<D, N>
{
    async fn intelligent_cross_modal_search(
        &self,
        query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Check if service is ready
        if !self.is_ready().await {
            let state = self.get_state().await;
            return Err(NodeSpaceError::InternalError {
                message: format!("Service not ready: {:?}", state),
                service: "core-logic".to_string(),
            });
        }

        // Step 1: Extract entities, temporal refs, and visual attributes
        let entities = self.extract_entities(query).await?;
        let temporal_refs = self.extract_temporal_refs(query).await?;
        let visual_refs = self.extract_visual_refs(query).await?;

        // Step 2: Generate query embedding
        let query_embedding = self.nlp_engine.generate_embedding(query).await?;

        // Step 3: Coordinate multiple search strategies
        let search_results = self
            .multi_strategy_search(query_embedding, &entities, &temporal_refs, &visual_refs)
            .await?;

        // Step 4: Intelligent fusion and ranking
        self.intelligent_result_fusion(search_results, query).await
    }

    async fn extract_entities(&self, query: &str) -> NodeSpaceResult<ExtractedEntities> {
        // Use LLM for entity extraction with structured prompt
        let extraction_prompt = format!(
            "Extract entities from this query and respond in JSON format:
Query: '{}'

Extract:
- people: names of people mentioned
- events: events or occasions mentioned  
- objects: physical objects or items mentioned
- locations: places or locations mentioned

Respond with JSON: {{\"people\": [...], \"events\": [...], \"objects\": [...], \"locations\": [...]}}",
            query
        );

        match self.nlp_engine.generate_text(&extraction_prompt).await {
            Ok(response) => {
                // Parse JSON response
                match serde_json::from_str::<ExtractedEntities>(&response) {
                    Ok(entities) => Ok(entities),
                    Err(_) => {
                        // Fallback: simple pattern matching if JSON parsing fails
                        Ok(self.extract_entities_fallback(query))
                    }
                }
            }
            Err(_) => {
                // Fallback to pattern matching if LLM fails
                Ok(self.extract_entities_fallback(query))
            }
        }
    }

    async fn extract_temporal_refs(&self, query: &str) -> NodeSpaceResult<Vec<TemporalReference>> {
        let temporal_prompt = format!(
            "Extract temporal references from this query. Look for dates, times, or event-based temporal references:
Query: '{}'

Find references like:
- Specific dates (June 15, 2023-06-15)
- Relative times (yesterday, last week)  
- Event-based times (during birthday, at the meeting)
- Fuzzy times (around that time, recently)

For each reference, determine if it's exact, relative, event-based, or fuzzy.",
            query
        );

        match self.nlp_engine.generate_text(&temporal_prompt).await {
            Ok(_response) => {
                // For MVP, do simple pattern matching
                Ok(self.extract_temporal_refs_fallback(query))
            }
            Err(_) => {
                // Fallback to pattern matching
                Ok(self.extract_temporal_refs_fallback(query))
            }
        }
    }

    async fn extract_visual_refs(&self, query: &str) -> NodeSpaceResult<VisualAttributes> {
        // Look for visual attributes in the query
        let colors = self.extract_colors(query);
        let objects = self.extract_visual_objects(query);
        let scene_types = self.extract_scene_types(query);
        let people_descriptions = self.extract_people_descriptions(query);

        Ok(VisualAttributes {
            colors,
            objects,
            scene_types,
            people_descriptions,
        })
    }

    async fn multi_strategy_search(
        &self,
        query_embedding: Vec<f32>,
        entities: &ExtractedEntities,
        temporal_refs: &[TemporalReference],
        visual_refs: &VisualAttributes,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        // Strategy 1: Semantic search
        match self
            .data_store
            .semantic_search_with_embedding(
                query_embedding.clone(),
                constants::DEFAULT_MULTI_STRATEGY_LIMIT,
            )
            .await
        {
            Ok(semantic_results) => {
                for (node, score) in semantic_results {
                    all_results.push(SearchResult {
                        node_id: node.id.clone(),
                        node,
                        score,
                    });
                }
            }
            Err(_) => {
                // Continue with other strategies if semantic search fails
            }
        }

        // Strategy 2: Entity-based search
        for person in &entities.people {
            if let Ok(entity_results) = self.search_by_entity(person).await {
                all_results.extend(entity_results);
            }
        }

        // Strategy 3: Temporal search
        for temporal_ref in temporal_refs {
            if let Ok(temporal_results) = self.search_by_temporal_ref(temporal_ref).await {
                all_results.extend(temporal_results);
            }
        }

        // Strategy 4: Visual search (if visual attributes found)
        if !visual_refs.colors.is_empty() || !visual_refs.objects.is_empty() {
            if let Ok(visual_results) = self.search_by_visual_attributes(visual_refs).await {
                all_results.extend(visual_results);
            }
        }

        Ok(all_results)
    }

    async fn intelligent_result_fusion(
        &self,
        mut search_results: Vec<SearchResult>,
        _original_query: &str,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        // Remove duplicates based on node ID (using string representation)
        search_results.sort_by(|a, b| a.node_id.to_string().cmp(&b.node_id.to_string()));
        search_results.dedup_by(|a, b| a.node_id == b.node_id);

        // Sort by score (highest first)
        search_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        search_results.truncate(constants::DEFAULT_FINAL_RESULT_LIMIT);

        Ok(search_results)
    }
}

impl<D: DataStore + Send + Sync, N: NLPEngine + Send + Sync> NodeSpaceService<D, N> {
    /// Fallback entity extraction using pattern matching
    fn extract_entities_fallback(&self, query: &str) -> ExtractedEntities {
        let query_lower = query.to_lowercase();
        let mut people = Vec::new();
        let mut events = Vec::new();
        let mut objects = Vec::new();
        let locations = Vec::new();

        // Common name patterns (basic detection)
        let name_patterns = ["claire", "john", "mary", "david", "sarah", "mike", "anna"];
        for pattern in &name_patterns {
            if query_lower.contains(pattern) {
                people.push(pattern.to_string());
            }
        }

        // Event patterns
        let event_patterns = [
            "birthday",
            "meeting",
            "party",
            "conference",
            "dinner",
            "lunch",
        ];
        for pattern in &event_patterns {
            if query_lower.contains(pattern) {
                events.push(pattern.to_string());
            }
        }

        // Object patterns
        let object_patterns = [
            "shirt",
            "document",
            "photo",
            "screenshot",
            "diagram",
            "chart",
        ];
        for pattern in &object_patterns {
            if query_lower.contains(pattern) {
                objects.push(pattern.to_string());
            }
        }

        ExtractedEntities {
            people,
            events,
            objects,
            locations,
        }
    }

    /// Fallback temporal reference extraction
    fn extract_temporal_refs_fallback(&self, query: &str) -> Vec<TemporalReference> {
        let query_lower = query.to_lowercase();
        let mut refs = Vec::new();

        // Look for relative time patterns
        if query_lower.contains("yesterday") {
            refs.push(TemporalReference {
                raw_text: "yesterday".to_string(),
                parsed_date: Some(chrono::Utc::now().date_naive() - chrono::Duration::days(1)),
                date_range: None,
                temporal_type: TemporalType::Relative,
            });
        }

        if query_lower.contains("birthday") {
            refs.push(TemporalReference {
                raw_text: "birthday".to_string(),
                parsed_date: None,
                date_range: None,
                temporal_type: TemporalType::Event,
            });
        }

        if query_lower.contains("last week") {
            let start_date = chrono::Utc::now().date_naive() - chrono::Duration::days(7);
            let end_date = chrono::Utc::now().date_naive();
            refs.push(TemporalReference {
                raw_text: "last week".to_string(),
                parsed_date: None,
                date_range: Some((start_date, end_date)),
                temporal_type: TemporalType::Relative,
            });
        }

        refs
    }

    /// Extract color references from query
    fn extract_colors(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let colors = [
            "red", "blue", "green", "yellow", "black", "white", "purple", "orange", "pink", "brown",
        ];

        colors
            .iter()
            .filter(|&color| query_lower.contains(color))
            .map(|&color| color.to_string())
            .collect()
    }

    /// Extract visual object references
    fn extract_visual_objects(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let objects = [
            "shirt", "dress", "hat", "car", "building", "tree", "person", "face",
        ];

        objects
            .iter()
            .filter(|&obj| query_lower.contains(obj))
            .map(|&obj| obj.to_string())
            .collect()
    }

    /// Extract scene type references
    fn extract_scene_types(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let scenes = [
            "indoor",
            "outdoor",
            "office",
            "restaurant",
            "home",
            "park",
            "street",
        ];

        scenes
            .iter()
            .filter(|&scene| query_lower.contains(scene))
            .map(|&scene| scene.to_string())
            .collect()
    }

    /// Extract people descriptions
    fn extract_people_descriptions(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut descriptions = Vec::new();

        if query_lower.contains("wearing") {
            descriptions.push("wearing clothing".to_string());
        }
        if query_lower.contains("smiling") {
            descriptions.push("smiling".to_string());
        }

        descriptions
    }

    /// Search nodes by entity mentions
    async fn search_by_entity(&self, entity: &str) -> NodeSpaceResult<Vec<SearchResult>> {
        // For LanceDB: Simple content search by entity
        let all_nodes = self.data_store.query_nodes(entity).await?;
        let nodes: Vec<_> = all_nodes
            .into_iter()
            .take(constants::DEFAULT_MAX_RESULTS_PER_STRATEGY)
            .collect();
        let results = nodes
            .into_iter()
            .enumerate()
            .map(|(index, node)| SearchResult {
                node_id: node.id.clone(),
                node,
                score: constants::BASE_CONFIDENCE_WITH_CONTEXT
                    - (index as f32 * constants::SCORE_DECAY_FACTOR * 0.5),
            })
            .collect();

        Ok(results)
    }

    /// Search nodes by temporal reference
    async fn search_by_temporal_ref(
        &self,
        temporal_ref: &TemporalReference,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        match &temporal_ref.temporal_type {
            TemporalType::Event => {
                // Search for event mentions
                self.search_by_entity(&temporal_ref.raw_text).await
            }
            TemporalType::Exact | TemporalType::Relative => {
                if let Some(date) = temporal_ref.parsed_date {
                    let date_str = date.format("%Y-%m-%d").to_string();
                    // For LanceDB: Get all nodes and filter by date
                    let all_nodes = self.data_store.query_nodes("").await?;
                    let nodes: Vec<_> = all_nodes
                        .into_iter()
                        .filter(|node| node.created_at.starts_with(&date_str))
                        .take(constants::DEFAULT_MAX_RESULTS_PER_STRATEGY)
                        .collect();
                    let results = nodes
                        .into_iter()
                        .enumerate()
                        .map(|(index, node)| SearchResult {
                            node_id: node.id.clone(),
                            node,
                            score: 0.9 - (index as f32 * constants::SCORE_DECAY_FACTOR * 0.5),
                        })
                        .collect();

                    Ok(results)
                } else {
                    Ok(Vec::new())
                }
            }
            TemporalType::Fuzzy => {
                // For fuzzy temporal references, return empty for now
                Ok(Vec::new())
            }
        }
    }

    /// Search nodes by visual attributes
    async fn search_by_visual_attributes(
        &self,
        visual_refs: &VisualAttributes,
    ) -> NodeSpaceResult<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Search for color mentions
        for color in &visual_refs.colors {
            if let Ok(color_results) = self.search_by_entity(color).await {
                results.extend(color_results);
            }
        }

        // Search for object mentions
        for object in &visual_refs.objects {
            if let Ok(object_results) = self.search_by_entity(object).await {
                results.extend(object_results);
            }
        }

        Ok(results)
    }

    /// Batch lookup for multiple node relationships (optimization for N+1 queries)
    async fn get_batch_related_nodes(
        &self,
        node_ids: &[NodeId],
        relationship_types: Vec<String>,
    ) -> NodeSpaceResult<std::collections::HashMap<NodeId, Vec<NodeId>>> {
        use std::collections::HashMap;

        if node_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Get all nodes once to avoid multiple database calls
        let all_nodes = self.data_store.query_nodes("").await.unwrap_or_default();
        let mut result_map = HashMap::new();

        // Initialize empty vectors for all requested node IDs
        for node_id in node_ids {
            result_map.insert(node_id.clone(), Vec::new());
        }

        // Process all nodes to find relationships
        for node in all_nodes {
            if let Some(metadata) = &node.metadata {
                // Check mentions
                if let Some(mentions) = metadata.get("mentions") {
                    if let Some(mentions_array) = mentions.as_array() {
                        for mention in mentions_array {
                            if let Some(mention_str) = mention.as_str() {
                                let mentioned_id = NodeId::from_string(mention_str.to_string());
                                if let Some(related_list) = result_map.get_mut(&mentioned_id) {
                                    related_list.push(node.id.clone());
                                }
                            }
                        }
                    }
                }

                // Check specific relationship types if provided
                if !relationship_types.is_empty() {
                    if let Some(relationships) = metadata.get("relationships") {
                        if let Some(rel_obj) = relationships.as_object() {
                            for rel_type in &relationship_types {
                                if let Some(targets) = rel_obj.get(rel_type) {
                                    if let Some(targets_array) = targets.as_array() {
                                        for target in targets_array {
                                            if let Some(target_str) = target.as_str() {
                                                let target_id =
                                                    NodeId::from_string(target_str.to_string());
                                                if let Some(related_list) =
                                                    result_map.get_mut(&target_id)
                                                {
                                                    related_list.push(node.id.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result_map)
    }

    /// Helper method to gather context for query processing
    async fn gather_query_context(
        &self,
        query: &str,
    ) -> NodeSpaceResult<(Vec<String>, Vec<NodeId>)> {
        let search_results = self
            .semantic_search(query, constants::DEFAULT_SEARCH_LIMIT)
            .await?;

        let context: Vec<String> = search_results
            .iter()
            .filter_map(|result| result.node.content.as_str().map(|s| s.to_string()))
            .collect();

        let sources: Vec<NodeId> = search_results
            .iter()
            .map(|result| result.node_id.clone())
            .collect();

        Ok((context, sources))
    }

    /// Helper method to build contextual prompt with length management
    fn build_contextual_prompt(&self, query: &str, context: &[String]) -> String {
        let context_text = context.join("\n\n");
        let max_context_len = self
            .config
            .performance_config
            .context_window
            .unwrap_or(constants::DEFAULT_CONTEXT_WINDOW)
            .saturating_sub(query.len() + constants::PROMPT_STRUCTURE_RESERVE);

        let truncated_context = if context_text.len() > max_context_len {
            format!("{}...", &context_text[..max_context_len])
        } else {
            context_text
        };

        if truncated_context.is_empty() {
            format!("Answer this question based on general knowledge: {}", query)
        } else {
            format!(
                "Based on the following context, answer the question: {}\n\nContext:\n{}",
                query, truncated_context
            )
        }
    }

    /// Helper method to generate contextual answer with fallback handling
    async fn generate_contextual_answer(
        &self,
        prompt: &str,
        sources: &[NodeId],
    ) -> NodeSpaceResult<String> {
        match self.nlp_engine.generate_text(prompt).await {
            Ok(text) => Ok(text),
            Err(e) => {
                // Handle text generation failure based on configuration
                match self.config.offline_config.offline_fallback {
                    OfflineFallback::Error => {
                        Err(NodeSpaceError::Processing(ProcessingError::model_error(
                            "core-logic",
                            "text-generation",
                            &format!("Text generation failed: {}", e)
                        )))
                    }
                    OfflineFallback::Stub => {
                        Ok("I apologize, but I'm currently unable to generate a response due to AI system limitations. Please try again later.".to_string())
                    }
                    OfflineFallback::Cache => {
                        if sources.is_empty() {
                            Ok("I found no relevant information to answer your question.".to_string())
                        } else {
                            Ok(format!("I found {} related documents but cannot generate a detailed response at this time. Please review the source materials directly.", sources.len()))
                        }
                    }
                }
            }
        }
    }

    /// Helper method to calculate response confidence
    fn calculate_response_confidence(&self, context: &[String], answer: &str) -> f32 {
        let base_confidence = if context.is_empty() {
            constants::BASE_CONFIDENCE_NO_CONTEXT
        } else {
            constants::BASE_CONFIDENCE_WITH_CONTEXT
        };

        if answer.contains("currently unable") || answer.contains("cannot generate") {
            base_confidence * constants::FALLBACK_CONFIDENCE_FACTOR
        } else {
            base_confidence
        }
    }

    /// Helper method to generate related queries
    fn generate_related_queries(&self, query: &str) -> Vec<String> {
        vec![
            format!(
                "What else about {}?",
                query.split_whitespace().last().unwrap_or("this topic")
            ),
            format!(
                "How does {} work?",
                query
                    .split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ]
    }

    /// Build hierarchical structure with OrderedNode format
    fn build_ordered_hierarchy<'a>(&'a self, parent_id: &'a NodeId, start_depth: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = NodeSpaceResult<Vec<OrderedNode>>> + Send + 'a>> {
        Box::pin(async move {
            let children = self.get_children(parent_id).await?;
            let mut ordered_children = Vec::new();
            
            for (index, child) in children.into_iter().enumerate() {
                let grandchildren = self.build_ordered_hierarchy(&child.id, start_depth + 1).await?;
                
                ordered_children.push(OrderedNode {
                    node: child,
                    children: grandchildren,
                    depth: start_depth,
                    sibling_index: index as u32,
                });
            }
            
            Ok(ordered_children)
        })
    }
}

// Include tests module
#[cfg(test)]
mod tests;
