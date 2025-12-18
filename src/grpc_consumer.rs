use std::any::Any;

use yellowstone_grpc_proto::geyser::SubscribeUpdate;

/// Trait for consuming gyser updates from a gRPC stream.
pub trait GrpcConsumer: Any + Send + Sync + std::fmt::Debug {
    /// Consumes a single update message.
    fn consume(&self, message: &SubscribeUpdate) -> Result<(), String>;
}
