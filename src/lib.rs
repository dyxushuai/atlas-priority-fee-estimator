//! Atlas Priority Fee Estimator
//!
//! This crate provides a service to estimate Solana priority fees based on
//! real-time Geyser data and historical slot information.

/// Error types for the priority fee estimator.
pub mod errors;
/// gRPC consumer trait and implementations.
pub mod grpc_consumer;
/// gRPC Geyser client implementation.
pub mod grpc_geyser;
/// Internal hashing utilities.
pub(crate) mod hash;
/// Data models for priority fees and calculations.
pub mod model;
/// Core priority fee tracking and estimation logic.
pub mod priority_fee;
/// Statistical calculation algorithms for priority fees.
pub mod priority_fee_calculation;
/// JSON-RPC server implementation.
pub mod rpc_server;
/// Thread-safe slot cache for tracking recent slots.
pub mod slot_cache;
/// Solana-specific utilities and RPC helpers.
pub mod solana;
