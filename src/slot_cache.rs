use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::hash::DashSet;
use queues::{CircularBuffer, IsQueue};
use solana_sdk::slot_history::Slot;
use tracing::error;

/// A thread-safe cache for tracking recent slots.
#[derive(Debug, Clone)]
pub struct SlotCache {
    slot_queue: Arc<RwLock<CircularBuffer<Slot>>>,
    slot_set: Arc<DashSet<Slot>>,
    // Fast-path for the common case: many txns share the same slot.
    last_seen_slot: Arc<AtomicU64>,
}

impl SlotCache {
    /// Creates a new SlotCache with the specified capacity.
    pub fn new(slot_cache_length: usize) -> Self {
        Self {
            slot_queue: Arc::new(RwLock::new(CircularBuffer::new(slot_cache_length))),
            slot_set: Arc::new(DashSet::default()),
            last_seen_slot: Arc::new(AtomicU64::new(u64::MAX)),
        }
    }

    // this pushes a new slot into the cache,
    // and returns the oldest slot if the cache
    /// Pushes a new slot into the cache and returns the oldest slot if the cache is at capacity.
    pub fn push_pop(&self, slot: Slot) -> Option<Slot> {
        if self.last_seen_slot.load(Ordering::Relaxed) == slot {
            return None;
        }
        if self.slot_set.contains(&slot) {
            self.last_seen_slot.store(slot, Ordering::Relaxed);
            return None;
        }
        match self.slot_queue.write() {
            Ok(mut slot_queue) => {
                if self.slot_set.contains(&slot) {
                    self.last_seen_slot.store(slot, Ordering::Relaxed);
                    return None;
                }

                match slot_queue.add(slot) {
                    Ok(maybe_oldest_slot) => {
                        if let Some(oldest_slot) = maybe_oldest_slot {
                            self.slot_set.remove(&oldest_slot);
                        }
                        self.slot_set.insert(slot);
                        self.last_seen_slot.store(slot, Ordering::Relaxed);
                        maybe_oldest_slot
                    }
                    Err(e) => {
                        error!("error adding slot to slot queue: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("error getting write lock on slot queue: {}", e);
                None
            }
        }
    }

    /// Copies all slots currently in the cache into the provided vector.
    pub fn copy_slots(&self, vec: &mut Vec<Slot>) {
        vec.extend(self.slot_set.iter().map(|v| *v));
    }

    /// Returns the number of slots currently in the cache.
    pub fn len(&self) -> usize {
        self.slot_set.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.slot_set.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import the SlotCache and necessary components

    #[test]
    fn test_push_pop() {
        // Create a SlotCache with a small length for testing
        let slot_cache = SlotCache::new(100);
        let mut i = 0;
        while i < 100 {
            assert_eq!(slot_cache.push_pop(i), None);
            i += 1;
        }
        // Now push one more and it should return the oldest (first inserted)
        assert_eq!(slot_cache.push_pop(101), Some(0));

        // Ensure duplicates are not added
        assert_eq!(slot_cache.push_pop(3), None); // Already exists, should not insert or pop

        // Ensure pushing repeatedly doesn't make the cache grow
        let mut i = 0;
        let len = slot_cache.len();
        while i < 100 {
            assert_eq!(slot_cache.len(), len);
            i += 1;
        }
    }

    #[test]
    fn test_copy() {
        // Create a SlotCache with a small length for testing
        let slot_cache = SlotCache::new(100);
        for i in 0..100 {
            assert_eq!(slot_cache.push_pop(i), None);
            assert_eq!(slot_cache.len(), (i + 1) as usize);
        }

        let mut vec: Vec<Slot> = Vec::new();
        slot_cache.copy_slots(&mut vec);
        vec.sort();
        assert_eq!(vec, (0..100).collect::<Vec<Slot>>());

        vec.clear();
        slot_cache.copy_slots(&mut vec);
        vec.sort();
        assert_eq!(vec, (0..100).collect::<Vec<Slot>>());
    }

    #[test]
    fn test_copy_reversed() {
        // Create a SlotCache with a small length for testing
        let slot_cache = SlotCache::new(100);
        for i in (0..100).rev() {
            assert_eq!(slot_cache.push_pop(i), None);
            assert_eq!(slot_cache.len(), 100 - i as usize, "{i}");
        }

        let mut vec: Vec<Slot> = Vec::new();
        slot_cache.copy_slots(&mut vec);
        vec.sort();
        assert_eq!(vec, (0..100).collect::<Vec<Slot>>());
    }
}
