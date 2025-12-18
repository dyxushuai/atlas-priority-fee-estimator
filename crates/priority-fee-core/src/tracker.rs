//! Priority Fee Tracker: Core tracking and estimation logic.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::mapref::entry::Entry;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::slot_history::Slot;
use statrs::statistics::{Distribution, OrderStatistics};

use crate::calculation::{Calculations, DataStats};
use crate::model::{
    Fees, MicroLamportPriorityFeeDetails, MicroLamportPriorityFeeEstimates, PriorityFeesBySlot,
    SlotPriorityFees,
};
use crate::slot_cache::SlotCache;

/// Tracks priority fees across slots and provides estimation methods.
#[derive(Debug, Clone)]
pub struct PriorityFeeTracker {
    priority_fees: Arc<PriorityFeesBySlot>,
    slot_cache: SlotCache,
}

impl PriorityFeeTracker {
    /// Creates a new PriorityFeeTracker with the specified slot cache length.
    pub fn new(slot_cache_length: usize) -> Self {
        Self {
            priority_fees: Arc::new(PriorityFeesBySlot::default()),
            slot_cache: SlotCache::new(slot_cache_length),
        }
    }

    /// Returns a reference to the underlying priority fees data.
    pub fn priority_fees(&self) -> &PriorityFeesBySlot {
        &self.priority_fees
    }

    /// Returns a reference to the slot cache.
    pub const fn slot_cache(&self) -> &SlotCache {
        &self.slot_cache
    }

    /// Pushes a priority fee for a transaction into the tracker.
    pub fn push_priority_fee_for_txn(
        &self,
        slot: Slot,
        accounts: Vec<Pubkey>,
        priority_fee: u64,
        is_vote: bool,
    ) {
        // Update the slot cache
        if let Some(oldest_slot) = self.slot_cache.push_pop(slot) {
            self.priority_fees.remove(&oldest_slot);
        }

        // Update or insert priority fees for this slot
        match self.priority_fees.entry(slot) {
            Entry::Occupied(mut entry) => {
                let slot_fees = entry.get_mut();
                slot_fees.fees.add_fee(priority_fee as f64, is_vote);
                for account in accounts {
                    slot_fees
                        .account_fees
                        .entry(account)
                        .and_modify(|fees| fees.add_fee(priority_fee as f64, is_vote))
                        .or_insert(Fees::new(priority_fee as f64, is_vote));
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(SlotPriorityFees::new(slot, accounts, priority_fee, is_vote));
            }
        }
    }

    /// Calculates priority fee estimates based on the provided calculation algorithm.
    pub fn calculate_priority_fee(
        &self,
        calculation: &Calculations,
    ) -> anyhow::Result<MicroLamportPriorityFeeEstimates> {
        let mut data: DataStats = calculation.get_priority_fee_estimates(&self.priority_fees)?;
        let estimates = estimate_max_values(&mut data, MicroLamportPriorityFeeEstimates::default());
        Ok(estimates)
    }

    /// Calculates detailed priority fee estimates and statistics.
    pub fn calculate_priority_fee_details(
        &self,
        calculation: &Calculations,
    ) -> anyhow::Result<(
        MicroLamportPriorityFeeEstimates,
        HashMap<String, MicroLamportPriorityFeeDetails>,
    )> {
        let mut data: DataStats = calculation.get_priority_fee_estimates(&self.priority_fees)?;
        let mut res = HashMap::new();
        for (key, fees) in data.iter_mut() {
            let estimates = MicroLamportPriorityFeeEstimates {
                min: fees.percentile(0),
                low: fees.percentile(25),
                medium: fees.percentile(50),
                high: fees.percentile(75),
                very_high: fees.percentile(95),
                unsafe_max: fees.percentile(100),
            };
            res.insert(
                key.to_string(),
                MicroLamportPriorityFeeDetails {
                    estimates,
                    mean: fees.mean().unwrap_or(f64::NAN),
                    stdev: fees.std_dev().unwrap_or(f64::NAN),
                    skew: fees.skewness().unwrap_or(f64::NAN),
                    count: fees.len(),
                },
            );
        }
        let estimates = estimate_max_values(&mut data, MicroLamportPriorityFeeEstimates::default());
        Ok((estimates, res))
    }
}

fn estimate_max_values(
    fees: &mut DataStats,
    mut estimates: MicroLamportPriorityFeeEstimates,
) -> MicroLamportPriorityFeeEstimates {
    for data in fees.values_mut() {
        let min = data.percentile(0);
        let low = data.percentile(25);
        let medium = data.percentile(50);
        let high = data.percentile(75);
        let very_high = data.percentile(95);
        let max = data.percentile(100);

        if min > estimates.min || estimates.min.is_nan() {
            estimates.min = min;
        }
        if low > estimates.low || estimates.low.is_nan() {
            estimates.low = low;
        }
        if medium > estimates.medium || estimates.medium.is_nan() {
            estimates.medium = medium;
        }
        if high > estimates.high || estimates.high.is_nan() {
            estimates.high = high;
        }
        if very_high > estimates.very_high || estimates.very_high.is_nan() {
            estimates.very_high = very_high;
        }
        if max > estimates.unsafe_max || estimates.unsafe_max.is_nan() {
            estimates.unsafe_max = max;
        }
    }
    estimates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_fee_tracker_basic() {
        let tracker = PriorityFeeTracker::new(10);

        let fees: Vec<f64> = (0..=100).map(|i| i as f64).collect();
        let accounts = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];

        for fee in &fees {
            tracker.push_priority_fee_for_txn(1, accounts.clone(), *fee as u64, false);
        }

        let calc = Calculations::new_calculation1(&accounts, false, false, &None);
        let estimates = tracker
            .calculate_priority_fee(&calc)
            .expect("calculation should succeed");

        assert_eq!(estimates.min, 0.0);
        assert_eq!(estimates.low, 25.0);
        assert_eq!(estimates.medium, 50.0);
        assert_eq!(estimates.high, 75.0);
        assert_eq!(estimates.very_high, 96.0);
        assert_eq!(estimates.unsafe_max, 100.0);
    }
}
