/// Rapidhash-based build hasher for fast in-memory maps/sets.
///
/// This is a performance choice (not cryptographic). If you need stronger HashDoS
/// resistance, prefer a slower, more robust hasher.
type RapidBuildHasher = rapidhash::fast::RandomState;

pub type DashMap<K, V> = dashmap::DashMap<K, V, RapidBuildHasher>;
pub type DashSet<K> = dashmap::DashSet<K, RapidBuildHasher>;
