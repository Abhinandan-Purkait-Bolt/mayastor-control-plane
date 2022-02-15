pub(crate) mod registry;
pub(super) mod service;
mod specs;
/// node watchdog to keep track of a node's liveness
pub(crate) mod watchdog;

use super::{core::registry::Registry, handler, impl_request_handler, CliArgs};
use common::{errors::SvcError, Service};
use common_lib::mbus_api::{v0::*, *};

use async_trait::async_trait;
use common_lib::types::v0::message_bus::{
    ChannelVs, GetBlockDevices, GetNodes, GetSpecs, GetStates, Register,
};
use grpc::operations::{node::server::NodeServer, registration::server::RegistrationServer};
use std::{convert::TryInto, marker::PhantomData, sync::Arc};

pub(crate) async fn configure(builder: Service) -> Service {
    let node_service = create_node_service(&builder).await;
    let node_grpc_service = NodeServer::new(Arc::new(node_service.clone()));
    let registration_service = RegistrationServer::new(Arc::new(node_service.clone()));
    builder
        .with_shared_state(node_service)
        .with_shared_state(node_grpc_service)
        .with_shared_state(registration_service)
        .with_channel(ChannelVs::Registry)
        .with_subscription(handler!(GetSpecs))
        .with_subscription(handler!(GetStates))
        .with_channel(ChannelVs::Node)
        .with_subscription(handler!(GetBlockDevices))
        .with_default_liveness()
}

async fn create_node_service(builder: &Service) -> service::Service {
    let registry = builder.get_shared_state::<Registry>().clone();
    let deadline = CliArgs::args().deadline.into();
    let request = CliArgs::args().request_timeout.into();
    let connect = CliArgs::args().connect_timeout.into();
    let service = service::Service::new(registry.clone(), deadline, request, connect);

    // attempt to reload the node state based on the specification
    for node in registry.specs().get_nodes() {
        service
            .register_state(&Register {
                id: node.id().clone(),
                grpc_endpoint: node.endpoint().to_string(),
            })
            .await;
    }

    service
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_lib::types::v0::{
        message_bus::{Liveness, Node, NodeId, NodeState, NodeStatus},
        store::node::{NodeLabels, NodeSpec},
    };
    use std::time::Duration;
    use testlib::ClusterBuilder;

    /// Get new `Node` from the given parameters
    fn new_node(id: NodeId, endpoint: String, status: NodeStatus) -> Node {
        Node::new(
            id.clone(),
            Some(NodeSpec::new(
                id.clone(),
                endpoint.clone(),
                NodeLabels::new(),
            )),
            Some(NodeState::new(id, endpoint, status)),
        )
    }

    #[tokio::test]
    async fn node() {
        let cluster = ClusterBuilder::builder()
            .with_rest(false)
            .with_agents(vec!["core"])
            .with_node_deadline("2s")
            .build()
            .await
            .unwrap();
        let bus_timeout = TimeoutOptions::default()
            .with_timeout(Duration::from_secs(1))
            .with_timeout_backoff(Duration::from_millis(100));

        let maya_name = cluster.node(0);
        let grpc = format!("{}:10124", cluster.node_ip(0));

        let nodes = GetNodes::default().request().await.unwrap();
        tracing::info!("Nodes: {:?}", nodes);
        assert_eq!(nodes.0.len(), 1);
        assert_eq!(
            nodes.0.first().unwrap(),
            &new_node(maya_name.clone(), grpc.clone(), NodeStatus::Online)
        );
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let nodes = GetNodes::default().request().await.unwrap();
        tracing::info!("Nodes: {:?}", nodes);
        assert_eq!(nodes.0.len(), 1);
        // still Online because the node is reachable via gRPC!
        assert_eq!(
            nodes.0.first().unwrap(),
            &new_node(maya_name.clone(), grpc.clone(), NodeStatus::Online)
        );

        cluster.composer().kill(maya_name.as_str()).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let nodes = GetNodes::default().request().await.unwrap();
        tracing::info!("Nodes: {:?}", nodes);
        assert_eq!(nodes.0.len(), 1);
        assert_eq!(
            nodes.0.first().unwrap(),
            &new_node(maya_name.clone(), grpc.clone(), NodeStatus::Offline)
        );
        cluster.composer().start(maya_name.as_str()).await.unwrap();

        let node = nodes.0.first().cloned().unwrap();
        cluster.restart_core().await;
        Liveness {}
            .request_on_ext(ChannelVs::Node, bus_timeout.clone())
            .await
            .unwrap();

        let nodes = GetNodes::default().request().await.unwrap();
        tracing::info!("Nodes: {:?}", nodes);
        assert_eq!(nodes.0.len(), 1);
        assert_eq!(
            nodes.0.first().unwrap(),
            &new_node(maya_name.clone(), grpc.clone(), NodeStatus::Online)
        );

        cluster.composer().stop(maya_name.as_str()).await.unwrap();
        cluster.restart_core().await;
        Liveness {}
            .request_on_ext(ChannelVs::Node, bus_timeout)
            .await
            .unwrap();

        let nodes = GetNodes::default().request().await.unwrap();
        tracing::info!("Nodes: {:?}", nodes);
        assert_eq!(nodes.0.len(), 1);
        assert_eq!(
            nodes.0.first().unwrap(),
            &Node::new(maya_name.clone(), node.spec().cloned(), None)
        );
    }
}
