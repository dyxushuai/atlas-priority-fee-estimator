//! Data Models: Priority fee types, estimation structures, etc.

use crate::hash::DashMap;
use serde::{Deserialize, Serialize};
use solana_sdk::clock::Slot;
use solana_sdk::pubkey::Pubkey;
use std::fmt::{Debug, Display, Formatter};

/// Priority levels for fee estimation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PriorityLevel {
    /// 0th percentile
    Min,
    /// 25th percentile
    Low,
    /// 50th percentile
    Medium,
    /// 75th percentile
    High,
    /// 95th percentile
    VeryHigh,
    /// 100th percentile
    UnsafeMax,
    /// 50th percentile (default)
    Default,
}

impl From<String> for PriorityLevel {
    fn from(s: String) -> Self {
        match s.trim().to_uppercase().as_str() {
            "NONE" => PriorityLevel::Min,
            "LOW" => PriorityLevel::Low,
            "MEDIUM" => PriorityLevel::Medium,
            "HIGH" => PriorityLevel::High,
            "VERY_HIGH" => PriorityLevel::VeryHigh,
            "UNSAFE_MAX" => PriorityLevel::UnsafeMax,
            _ => PriorityLevel::Default,
        }
    }
}

impl From<PriorityLevel> for Percentile {
    fn from(val: PriorityLevel) -> Self {
        match val {
            PriorityLevel::Min => 0,
            PriorityLevel::Low => 25,
            PriorityLevel::Medium => 50,
            PriorityLevel::High => 75,
            PriorityLevel::VeryHigh => 95,
            PriorityLevel::UnsafeMax => 100,
            PriorityLevel::Default => 50,
        }
    }
}

/// Type alias for percentile values (0-100).
pub type Percentile = usize;

/// Types of data that can be used for priority fee calculations.
#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub enum DataType<'a> {
    /// Global fees across all transactions.
    Global,
    /// Fees across all specified accounts.
    AllAccounts,
    /// Fees for a specific account.
    Account(&'a Pubkey),
}

impl Display for DataType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Global => f.write_str("Global"),
            DataType::AllAccounts => f.write_str("All Accounts"),
            DataType::Account(pubkey) => f.write_str(pubkey.to_string().as_str()),
        }
    }
}

/// Priority fee estimates in micro-lamports for different priority levels.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct MicroLamportPriorityFeeEstimates {
    /// Minimum fee (0th percentile).
    pub min: f64,
    /// Low fee (25th percentile).
    pub low: f64,
    /// Medium fee (50th percentile).
    pub medium: f64,
    /// High fee (75th percentile).
    pub high: f64,
    /// Very high fee (95th percentile).
    pub very_high: f64,
    /// Unsafe maximum fee (100th percentile).
    pub unsafe_max: f64,
}

/// Detailed priority fee statistics.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct MicroLamportPriorityFeeDetails {
    /// Percentile estimates.
    pub estimates: MicroLamportPriorityFeeEstimates,
    /// Mean fee.
    pub mean: f64,
    /// Standard deviation of fees.
    pub stdev: f64,
    /// Skewness of fee distribution.
    pub skew: f64,
    /// Number of transactions included in the calculation.
    pub count: usize,
}

/// Collection of fees for a slot or account.
#[derive(Debug, Clone)]
pub struct Fees {
    /// Non-vote transaction fees.
    pub non_vote_fees: Vec<f64>,
    /// Vote transaction fees.
    pub vote_fees: Vec<f64>,
}

impl Fees {
    /// Creates a new Fees instance with an initial fee.
    pub fn new(fee: f64, is_vote: bool) -> Self {
        if is_vote {
            Self {
                vote_fees: vec![fee],
                non_vote_fees: vec![],
            }
        } else {
            Self {
                vote_fees: vec![],
                non_vote_fees: vec![fee],
            }
        }
    }

    /// Adds a fee to the collection.
    pub fn add_fee(&mut self, fee: f64, is_vote: bool) {
        if is_vote {
            self.vote_fees.push(fee);
        } else {
            self.non_vote_fees.push(fee);
        }
    }
}

/// Priority fees for a specific slot.
#[derive(Debug, Clone)]
pub struct SlotPriorityFees {
    /// Slot number.
    pub slot: Slot,
    /// Global fees for the slot.
    pub fees: Fees,
    /// Per-account fees for the slot.
    pub account_fees: DashMap<Pubkey, Fees>,
}

impl SlotPriorityFees {
    /// Creates a new SlotPriorityFees instance.
    pub fn new(slot: Slot, accounts: Vec<Pubkey>, priority_fee: u64, is_vote: bool) -> Self {
        let account_fees = DashMap::default();
        let fees = Fees::new(priority_fee as f64, is_vote);
        for account in accounts {
            account_fees.insert(account, fees.clone());
        }
        Self {
            slot,
            fees,
            account_fees,
        }
    }
}

/// Type alias for a thread-safe map of priority fees by slot.
pub type PriorityFeesBySlot = DashMap<Slot, SlotPriorityFees>;
