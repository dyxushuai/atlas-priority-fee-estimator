//! Atlas Priority Fee Estimator Binary
//!
//! This binary starts the priority fee estimator service, which consumes
//! Solana Geyser data via gRPC and provides a JSON-RPC API for fee estimation.

use std::{env, net::UdpSocket, sync::Arc};

use atlas_priority_fee_estimator::grpc_geyser::GrpcGeyserImpl;
use atlas_priority_fee_estimator::priority_fee::PriorityFeeTracker;
use atlas_priority_fee_estimator::rpc_server::{
    AtlasPriorityFeeEstimator, AtlasPriorityFeeEstimatorRpcServer,
};
use cadence::{BufferedUdpMetricSink, QueuingMetricSink, StatsdClient};
use cadence_macros::set_global_default;
use figment::{providers::Env, Figment};
use jsonrpsee::server::middleware::http::ProxyGetRequestLayer;
use jsonrpsee::server::{ServerBuilder, ServerConfig};
use serde::Deserialize;
use tracing::{error, info};

#[derive(Debug, Deserialize, Clone)]
struct EstimatorEnv {
    max_lookback_slots: Option<usize>,
    port: Option<u16>,
    grpc_url: String,
    grpc_x_token: Option<String>,
    rpc_url: String,
}

#[tokio::main]
async fn main() {
    // Init metrics/logging
    let env: EstimatorEnv = Figment::from(Env::raw()).extract().unwrap();
    let env_filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .init();
    new_metrics_client();
    let max_lookback_slots = env.max_lookback_slots.unwrap_or(150);
    let priority_fee_tracker = Arc::new(PriorityFeeTracker::new(max_lookback_slots));
    // start grpc consumer
    let _ = GrpcGeyserImpl::new(
        env.grpc_url,
        env.grpc_x_token,
        vec![priority_fee_tracker.clone()],
    );

    let port = env.port.unwrap_or(4141);
    let config = ServerConfig::builder().max_connections(100_000).build();
    let server = ServerBuilder::with_config(config)
        .set_http_middleware(
            tower::ServiceBuilder::new()
                // Proxy `GET /health` requests to internal `health` method.
                .layer(
                    ProxyGetRequestLayer::new([("/health", "health")])
                        .expect("expected health check to initialize"),
                ),
        )
        .build(format!("0.0.0.0:{}", port))
        .await
        .unwrap_or_else(|_| panic!("failed to start server on port {}", port));
    let rpc = AtlasPriorityFeeEstimator::new(priority_fee_tracker, env.rpc_url, max_lookback_slots);
    let handle = server.start(rpc.into_rpc());
    handle.stopped().await;
}

fn new_metrics_client() {
    let uri = env::var("METRICS_URI").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("METRICS_PORT")
        .unwrap_or_else(|_| "7998".to_string())
        .parse::<u16>()
        .unwrap();
    info!("collecting metrics on: {}:{}", uri, port);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_nonblocking(true).unwrap();

    let host = (uri, port);
    let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    let queuing_sink = QueuingMetricSink::from(udp_sink);
    let builder = StatsdClient::builder("atlas_priority_fee_estimator", queuing_sink);
    let client = builder
        .with_error_handler(|e| error!("statsd metrics error: {}", e))
        .build();
    set_global_default(client);
}
