//! Atlas Priority Fee Estimator
//!
//! This crate provides a service to estimate Solana priority fees based on
//! real-time Geyser data and historical slot information.

// Re-export core types from priority-fee-core
pub use priority_fee_core::{
    Calculations, DataType, Fees, MicroLamportPriorityFeeDetails, MicroLamportPriorityFeeEstimates,
    PriorityFeeTracker, PriorityFeesBySlot, PriorityLevel, SlotCache, SlotPriorityFees,
};

/// Error types for the priority fee estimator.
pub mod errors;
/// gRPC consumer trait and implementations.
pub mod grpc_consumer;
/// gRPC Geyser client implementation.
pub mod grpc_geyser;
/// Core priority fee tracking and estimation logic with gRPC integration.
pub mod priority_fee;
/// Statistical calculation algorithms with metrics integration.
pub mod priority_fee_calculation;
/// JSON-RPC server implementation.
pub mod rpc_server;
/// Solana-specific utilities and RPC helpers.
pub mod solana;

// Re-exports for model types are now from priority-fee-core
// slot_cache is also from priority-fee-core
