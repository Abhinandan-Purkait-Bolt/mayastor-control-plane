mod registry;
pub mod service;
pub mod specs;
use http::Uri;
use std::sync::Arc;
use tonic::transport::Server;

use super::core::registry::Registry;

use common::{handler::*, Service};
use grpc::{pool::server::PoolServer, replica::server::ReplicaServer};

pub(crate) fn configure(builder: Service) -> Service {
    let registry = builder.get_shared_state::<Registry>().clone();
    let new_service = service::Service::new(registry.clone());
    builder
        .with_channel(ChannelVs::Pool)
        .with_default_liveness()
        .with_shared_state(new_service.clone())
        .add_grpc_service(PoolGrpcServer::new(PoolServer::new(pool_service)))
        .add_grpc_service(ReplicaGrpcServer::new(ReplicaServer::new(replica_service)))
}

/// Pool Agent's Tests
#[cfg(test)]
mod tests;
