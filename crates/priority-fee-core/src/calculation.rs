//! Calculation Algorithms: v1/v2 percentile algorithms.

use crate::model::{DataType, PriorityFeesBySlot};
use solana_sdk::pubkey::Pubkey;
use statrs::statistics::Data;
use std::collections::HashMap;
use Calculations::{Calculation1, Calculation2};

/// The result type for priority fee statistics.
pub type DataStats<'a> = HashMap<DataType<'a>, Data<Vec<f64>>>;

/// Enum representing different priority fee calculation algorithms.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Calculations<'a> {
    /// Algorithm 1: Collects all transaction fees and fees for specified accounts over n slots.
    Calculation1 {
        /// Accounts to filter fees for.
        accounts: &'a [Pubkey],
        /// Whether to include vote transactions.
        include_vote: bool,
        /// Whether to include empty slots in the calculation.
        include_empty_slots: bool,
        /// Lookback period in slots.
        lookback_period: &'a Option<u32>,
    },
    /// Algorithm 2: Collects all transaction fees and fees for each specified account separately.
    Calculation2 {
        /// Accounts to filter fees for.
        accounts: &'a [Pubkey],
        /// Whether to include vote transactions.
        include_vote: bool,
        /// Whether to include empty slots in the calculation.
        include_empty_slots: bool,
        /// Lookback period in slots.
        lookback_period: &'a Option<u32>,
    },
}

impl<'a> Calculations<'a> {
    /// Creates a new Calculation1 instance.
    pub const fn new_calculation1(
        accounts: &'a [Pubkey],
        include_vote: bool,
        include_empty_slots: bool,
        lookback_period: &'a Option<u32>,
    ) -> Calculations<'a> {
        Calculation1 {
            accounts,
            include_vote,
            include_empty_slots,
            lookback_period,
        }
    }

    /// Creates a new Calculation2 instance.
    pub const fn new_calculation2(
        accounts: &'a [Pubkey],
        include_vote: bool,
        include_empty_slots: bool,
        lookback_period: &'a Option<u32>,
    ) -> Calculations<'a> {
        Calculation2 {
            accounts,
            include_vote,
            include_empty_slots,
            lookback_period,
        }
    }

    /// Calculates priority fee estimates based on the selected algorithm.
    pub fn get_priority_fee_estimates(
        &self,
        priority_fees: &PriorityFeesBySlot,
    ) -> anyhow::Result<DataStats<'a>> {
        match self {
            Calculation1 {
                accounts,
                include_vote,
                include_empty_slots,
                lookback_period,
            } => v1::get_priority_fee_estimates(
                accounts,
                include_vote,
                include_empty_slots,
                lookback_period,
                priority_fees,
            ),
            Calculation2 {
                accounts,
                include_vote,
                include_empty_slots,
                lookback_period,
            } => v2::get_priority_fee_estimates(
                accounts,
                include_vote,
                include_empty_slots,
                lookback_period,
                priority_fees,
            ),
        }
    }
}

mod v1 {
    use super::{calculate_lookback_size, DataStats, DataType};
    use crate::model::PriorityFeesBySlot;
    use solana_sdk::clock::Slot;
    use solana_sdk::pubkey::Pubkey;
    use statrs::statistics::Data;

    /// Algorithm 1: Collects all transaction fees and fees for all specified accounts.
    pub(super) fn get_priority_fee_estimates<'a>(
        accounts: &'a [Pubkey],
        include_vote: &bool,
        include_empty_slots: &bool,
        lookback_period: &Option<u32>,
        priority_fees: &PriorityFeesBySlot,
    ) -> anyhow::Result<DataStats<'a>> {
        let mut slots_vec: Vec<Slot> = priority_fees.iter().map(|entry| entry.slot).collect();
        slots_vec.sort();
        slots_vec.reverse();

        let lookback = calculate_lookback_size(lookback_period, slots_vec.len());

        let mut global_fees: Vec<f64> = Vec::new();
        let mut account_fees: Vec<f64> = Vec::new();
        for slot in &slots_vec[..lookback] {
            if let Some(slot_priority_fees) = priority_fees.get(slot) {
                if *include_vote {
                    global_fees.extend_from_slice(&slot_priority_fees.fees.vote_fees);
                }
                global_fees.extend_from_slice(&slot_priority_fees.fees.non_vote_fees);

                if !accounts.is_empty() {
                    let mut has_data = false;
                    accounts.iter().for_each(|account| {
                        if let Some(account_priority_fees) =
                            slot_priority_fees.account_fees.get(account)
                        {
                            if *include_vote {
                                account_fees.extend_from_slice(&account_priority_fees.vote_fees);
                            }
                            account_fees.extend_from_slice(&account_priority_fees.non_vote_fees);
                            has_data = true;
                        }
                    });
                    if !has_data && *include_empty_slots {
                        account_fees.push(0f64);
                    }
                }
            }
        }

        let mut data = DataStats::new();
        data.insert(DataType::Global, Data::new(global_fees));
        data.insert(DataType::AllAccounts, Data::new(account_fees));
        Ok(data)
    }
}

mod v2 {
    use super::{calculate_lookback_size, DataStats, DataType};
    use crate::model::PriorityFeesBySlot;
    use solana_sdk::clock::Slot;
    use solana_sdk::pubkey::Pubkey;
    use statrs::statistics::Data;
    use std::collections::HashMap;

    /// Algorithm 2: Collects fees for each specified account separately.
    pub(super) fn get_priority_fee_estimates<'a>(
        accounts: &'a [Pubkey],
        include_vote: &bool,
        include_empty_slots: &bool,
        lookback_period: &Option<u32>,
        priority_fees: &PriorityFeesBySlot,
    ) -> anyhow::Result<DataStats<'a>> {
        let mut slots_vec: Vec<Slot> = priority_fees.iter().map(|entry| entry.slot).collect();
        slots_vec.sort();
        slots_vec.reverse();

        let lookback = calculate_lookback_size(lookback_period, slots_vec.len());

        let mut data: HashMap<DataType<'a>, Vec<f64>> = HashMap::new();
        for slot in &slots_vec[..lookback] {
            if let Some(slot_priority_fees) = priority_fees.get(slot) {
                let fees: &mut Vec<f64> = data.entry(DataType::Global).or_default();

                if *include_vote {
                    fees.extend_from_slice(&slot_priority_fees.fees.vote_fees);
                }
                fees.extend_from_slice(&slot_priority_fees.fees.non_vote_fees);

                accounts.iter().for_each(|account| {
                    let fees: &mut Vec<f64> = data.entry(DataType::Account(account)).or_default();
                    if let Some(account_priority_fees) =
                        slot_priority_fees.account_fees.get(account)
                    {
                        if *include_vote {
                            fees.extend_from_slice(&account_priority_fees.vote_fees);
                        }
                        fees.extend_from_slice(&account_priority_fees.non_vote_fees);
                    } else if *include_empty_slots {
                        fees.push(0f64);
                    }
                });
            }
        }

        let data = data
            .into_iter()
            .map(|(data_type, fees)| (data_type, Data::new(fees)))
            .collect::<DataStats>();
        Ok(data)
    }
}

fn calculate_lookback_size(pref_num_slots: &Option<u32>, max_available_slots: usize) -> usize {
    max_available_slots.min(
        pref_num_slots
            .map(|v| v as usize)
            .unwrap_or(max_available_slots),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculation::DataType::{Account, AllAccounts, Global};
    use crate::model::{Fees, SlotPriorityFees};
    use solana_sdk::clock::Slot;
    use statrs::statistics::OrderStatistics;

    #[test]
    fn test_specific_fee_estimates_for_global_accounts_only() {
        let tracker = PriorityFeesBySlot::default();

        let fees: Vec<f64> = (0..=100).map(|i| i as f64).collect();
        let account_1 = Pubkey::new_unique();
        let account_2 = Pubkey::new_unique();
        let account_3 = Pubkey::new_unique();
        let accounts = vec![account_1, account_2, account_3];

        for fee in &fees {
            push_priority_fee_for_txn(1, accounts.clone(), *fee as u64, false, &tracker);
        }

        let empty_accounts: Vec<Pubkey> = vec![];
        let calc = Calculations::new_calculation1(&empty_accounts, false, false, &None);
        let mut estimates: DataStats = calc
            .get_priority_fee_estimates(&tracker)
            .expect("estimates to be valid");

        assert_eq!(estimates.len(), 2);
        {
            let stats = estimates.get_mut(&Global).unwrap();
            assert_eq!(stats.percentile(50).round(), 50.0);
        }
    }

    fn push_priority_fee_for_txn(
        slot: Slot,
        accounts: Vec<Pubkey>,
        priority_fee: u64,
        is_vote: bool,
        priority_fees: &PriorityFeesBySlot,
    ) {
        if !priority_fees.contains_key(&slot) {
            priority_fees.insert(
                slot,
                SlotPriorityFees::new(slot, accounts, priority_fee, is_vote),
            );
        } else {
            priority_fees.entry(slot).and_modify(|priority_fees| {
                priority_fees.fees.add_fee(priority_fee as f64, is_vote);
                for account in accounts {
                    priority_fees
                        .account_fees
                        .entry(account)
                        .and_modify(|fees| fees.add_fee(priority_fee as f64, is_vote))
                        .or_insert(Fees::new(priority_fee as f64, is_vote));
                }
            });
        }
    }
}
