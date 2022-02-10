use crate::{
    common::{
        NodeFilter, NodePoolFilter, NodePoolReplicaFilter, NodeReplicaFilter, PoolFilter,
        PoolReplicaFilter, ReplicaFilter, VolumeFilter,
    },
    operations::replica::traits::ReplicaOperations,
    replica::{
        create_replica_reply, get_replicas_reply, get_replicas_request,
        replica_grpc_client::ReplicaGrpcClient, share_replica_reply, GetReplicasRequest,
    },
};

use std::{convert::TryFrom, ops::Deref, time::Duration};
use tonic::transport::{Channel, Endpoint, Uri};

use crate::{
    grpc_opts::{Client, Context},
    operations::replica::traits::{
        CreateReplicaInfo, DestroyReplicaInfo, ShareReplicaInfo, UnshareReplicaInfo,
    },
};
use common_lib::{
    mbus_api::{v0::Replicas, ReplyError, ResourceKind, TimeoutOptions},
    types::v0::message_bus::{Filter, MessageIdVs, Replica},
};

/// RPC Replica Client
#[derive(Clone)]
pub struct ReplicaClient {
    inner: Client<ReplicaGrpcClient<Channel>>,
}
impl Deref for ReplicaClient {
    type Target = Client<ReplicaGrpcClient<Channel>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl ReplicaClient {
    /// creates a new base tonic endpoint with the timeout options and the address
    pub async fn new<O: Into<Option<TimeoutOptions>>>(addr: Uri, opts: O) -> Self {
        let client = Client::new(addr, opts, ReplicaGrpcClient::new).await;
        Self { inner: client }
    }
}

#[tonic::async_trait]
impl ReplicaOperations for ReplicaClient {
    async fn create(
        &self,
        create_replica_req: &dyn CreateReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<Replica, ReplyError> {
        let req = self.request(create_replica_req, ctx, MessageIdVs::CreateReplica);
        let response = self.client().create_replica(req).await?.into_inner();
        match response.reply {
            Some(create_replica_reply) => match create_replica_reply {
                create_replica_reply::Reply::Replica(replica) => Ok(Replica::try_from(replica)?),
                create_replica_reply::Reply::Error(err) => Err(err.into()),
            },
            None => Err(ReplyError::invalid_response(ResourceKind::Replica)),
        }
    }

    async fn get(&self, filter: Filter, ctx: Option<Context>) -> Result<Replicas, ReplyError> {
        let req: GetReplicasRequest = match filter {
            Filter::Node(id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::Node(NodeFilter {
                    node_id: id.into(),
                })),
            },
            Filter::Pool(id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::Pool(PoolFilter {
                    pool_id: id.into(),
                })),
            },
            Filter::NodePool(node_id, pool_id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::NodePool(NodePoolFilter {
                    node_id: node_id.into(),
                    pool_id: pool_id.into(),
                })),
            },
            Filter::NodePoolReplica(node_id, pool_id, replica_id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::NodePoolReplica(
                    NodePoolReplicaFilter {
                        node_id: node_id.into(),
                        pool_id: pool_id.into(),
                        replica_id: replica_id.to_string(),
                    },
                )),
            },
            Filter::NodeReplica(node_id, replica_id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::NodeReplica(
                    NodeReplicaFilter {
                        node_id: node_id.into(),
                        replica_id: replica_id.to_string(),
                    },
                )),
            },
            Filter::PoolReplica(pool_id, replica_id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::PoolReplica(
                    PoolReplicaFilter {
                        pool_id: pool_id.into(),
                        replica_id: replica_id.to_string(),
                    },
                )),
            },
            Filter::Replica(replica_id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::Replica(ReplicaFilter {
                    replica_id: replica_id.to_string(),
                })),
            },
            Filter::Volume(volume_id) => GetReplicasRequest {
                filter: Some(get_replicas_request::Filter::Volume(VolumeFilter {
                    volume_id: volume_id.to_string(),
                })),
            },
            _ => GetReplicasRequest { filter: None },
        };
        let req = self.request(req, ctx, MessageIdVs::GetReplicas);
        let response = self.client().get_replicas(req).await?.into_inner();
        match response.reply {
            Some(get_replicas_reply) => match get_replicas_reply {
                get_replicas_reply::Reply::Replicas(replicas) => Ok(Replicas::try_from(replicas)?),
                get_replicas_reply::Reply::Error(err) => Err(err.into()),
            },
            None => Err(ReplyError::invalid_response(ResourceKind::Replica)),
        }
    }

    async fn destroy(
        &self,
        destroy_replica_req: &dyn DestroyReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<(), ReplyError> {
        let req = self.request(destroy_replica_req, ctx, MessageIdVs::DestroyReplica);
        let response = self.client().destroy_replica(req).await?.into_inner();
        match response.error {
            None => Ok(()),
            Some(err) => Err(err.into()),
        }
    }

    async fn share(
        &self,
        share_replica_req: &dyn ShareReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<String, ReplyError> {
        let req = self.request(share_replica_req, ctx, MessageIdVs::ShareReplica);
        let response = self.client().share_replica(req).await?.into_inner();
        match response.reply {
            Some(share_replica_reply) => match share_replica_reply {
                share_replica_reply::Reply::Response(message) => Ok(message),
                share_replica_reply::Reply::Error(err) => Err(err.into()),
            },
            None => Err(ReplyError::invalid_response(ResourceKind::Replica)),
        }
    }

    async fn unshare(
        &self,
        unshare_replica_req: &dyn UnshareReplicaInfo,
        ctx: Option<Context>,
    ) -> Result<(), ReplyError> {
        let req = self.request(unshare_replica_req, ctx, MessageIdVs::UnshareReplica);
        let response = self.client().unshare_replica(req).await?.into_inner();
        match response.error {
            None => Ok(()),
            Some(err) => Err(err.into()),
        }
    }
}
