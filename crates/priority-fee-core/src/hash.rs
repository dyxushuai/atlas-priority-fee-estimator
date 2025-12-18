//! Rapidhash-based build hasher for fast in-memory maps/sets.
//!
//! This is a performance choice (not cryptographic). If you need stronger HashDoS
//! resistance, prefer a slower, more robust hasher.

/// Rapidhash-based build hasher for fast in-memory maps/sets.
type RapidBuildHasher = rapidhash::fast::RandomState;

/// Thread-safe hash map using Rapidhash.
#[allow(unreachable_pub)]
pub type DashMap<K, V> = dashmap::DashMap<K, V, RapidBuildHasher>;

/// Thread-safe hash set using Rapidhash.
#[allow(unreachable_pub)]
pub type DashSet<K> = dashmap::DashSet<K, RapidBuildHasher>;
