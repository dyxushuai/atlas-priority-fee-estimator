/// Rapidhash-based build hasher for fast in-memory maps/sets.
///
/// This is a performance choice (not cryptographic). If you need stronger HashDoS
/// resistance, prefer a slower, more robust hasher.
type RapidBuildHasher = rapidhash::fast::RandomState;

/// Thread-safe hash map using Rapidhash.
#[allow(clippy::redundant_pub_crate)]
pub(crate) type DashMap<K, V> = dashmap::DashMap<K, V, RapidBuildHasher>;
/// Thread-safe hash set using Rapidhash.
#[allow(clippy::redundant_pub_crate)]
pub(crate) type DashSet<K> = dashmap::DashSet<K, RapidBuildHasher>;
