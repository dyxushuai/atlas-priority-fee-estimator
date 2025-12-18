//! Solana Priority Fee Core
//!
//! Core priority fee estimation algorithm library, can be used independently without gRPC/API layers.
//!
//! # Main Components
//!
//! - [`PriorityFeeTracker`] - Tracks and estimates priority fees
//! - [`Calculations`] - Calculation algorithms (v1/v2)
//! - [`SlotCache`] - Thread-safe slot cache
//!
//! # Usage Example
//!
//! ```ignore
//! use priority_fee_core::{PriorityFeeTracker, Calculations};
//!
//! let tracker = PriorityFeeTracker::new(150);
//! tracker.push_priority_fee_for_txn(slot, accounts, priority_fee, is_vote);
//! let estimates = tracker.calculate_priority_fee(&calculation)?;
//! ```

pub(crate) mod hash;

/// Data models: priority fee types, estimation structures, etc.
pub mod model;

/// Slot cache: thread-safe slot tracking.
pub mod slot_cache;

/// Calculation algorithms: v1/v2 percentile algorithms.
pub mod calculation;

/// Priority fee tracker: core tracking and estimation logic.
pub mod tracker;

// Re-export common types
pub use calculation::Calculations;
pub use model::{
    DataType, Fees, MicroLamportPriorityFeeDetails, MicroLamportPriorityFeeEstimates,
    PriorityFeesBySlot, PriorityLevel, SlotPriorityFees,
};
pub use slot_cache::SlotCache;
pub use tracker::PriorityFeeTracker;
