mod registry;
pub mod service;
pub mod specs;

use http::Uri;
use std::{net::SocketAddr, sync::Arc};

use super::core::registry::Registry;

use common::{handler::*, Service};
use grpc::{pool::server::PoolServer, replica::server::ReplicaServer};

pub(crate) fn configure(builder: Service) -> Service {
    let registry = builder.get_shared_state::<Registry>().clone();
    let grpc_addr: SocketAddr = builder
        .get_shared_state::<Uri>()
        .clone()
        .authority()
        .unwrap()
        .to_string()
        .parse()
        .unwrap();
    let new_service = Arc::new(service::Service::new(registry));
    let pool_service = PoolServer::new(new_service.clone()).into_grpc_server();
    let replica_service = ReplicaServer::new(new_service.clone()).into_grpc_server();
    builder
        .with_channel(ChannelVs::Pool)
        .with_default_liveness()
        .with_shared_state(new_service)
        .add_grpc_service(move |server| Box::pin(server.add_service(pool_service).serve(grpc_addr)))
        .add_grpc_service(move |server| {
            Box::pin(server.add_service(replica_service).serve(grpc_addr))
        })
}

/// Pool Agent's Tests
#[cfg(test)]
mod tests;
