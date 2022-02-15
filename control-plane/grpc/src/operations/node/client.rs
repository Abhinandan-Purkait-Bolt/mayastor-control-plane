use crate::{
    common::NodeFilter,
    grpc_opts::{Client, Context},
    node::{get_nodes_reply, get_nodes_request, node_grpc_client::NodeGrpcClient, GetNodesRequest},
    operations::node::traits::NodeOperations,
};
use common_lib::{
    mbus_api::{v0::Nodes, ReplyError, ResourceKind, TimeoutOptions},
    types::v0::message_bus::{Filter, MessageIdVs},
};
use std::{convert::TryFrom, ops::Deref};
use tonic::transport::{Channel, Uri};

/// RPC Pool Client
#[derive(Clone)]
pub struct NodeClient {
    inner: Client<NodeGrpcClient<Channel>>,
}
impl Deref for NodeClient {
    type Target = Client<NodeGrpcClient<Channel>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeClient {
    /// creates a new base tonic endpoint with the timeout options and the address
    pub async fn new<O: Into<Option<TimeoutOptions>>>(addr: Uri, opts: O) -> Self {
        let client = Client::new(addr, opts, NodeGrpcClient::new).await;
        Self { inner: client }
    }
}

#[tonic::async_trait]
impl NodeOperations for NodeClient {
    async fn get(&self, filter: Filter, ctx: Option<Context>) -> Result<Nodes, ReplyError> {
        let req: GetNodesRequest = match filter {
            Filter::Node(id) => GetNodesRequest {
                filter: Some(get_nodes_request::Filter::Node(NodeFilter {
                    node_id: id.into(),
                })),
            },
            _ => GetNodesRequest { filter: None },
        };
        let req = self.request(req, ctx, MessageIdVs::GetNodes);
        let response = self.client().get_nodes(req).await?.into_inner();
        match response.reply {
            Some(get_nodes_reply) => match get_nodes_reply {
                get_nodes_reply::Reply::Nodes(nodes) => Ok(Nodes::try_from(nodes)?),
                get_nodes_reply::Reply::Error(err) => Err(err.into()),
            },
            None => Err(ReplyError::invalid_response(ResourceKind::Pool)),
        }
    }
}
