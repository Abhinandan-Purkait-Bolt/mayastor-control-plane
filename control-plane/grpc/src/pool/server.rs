pub use crate::pool_grpc::pool_grpc_server::PoolGrpcServer;
use crate::{
    pool::traits::PoolOperations,
    pool_grpc,
    pool_grpc::{
        create_pool_reply, get_pools_reply, pool_grpc_server::PoolGrpc, CreatePoolReply,
        CreatePoolRequest, DestroyPoolReply, DestroyPoolRequest, GetPoolsReply, GetPoolsRequest,
    },
};
use common_lib::mbus_api::ReplyError;
use std::sync::Arc;
use tonic::{Request, Response};

use common_lib::types::v0::message_bus::Filter;
// RPC Pool Server
pub struct PoolServer {
    // Service which executes the operations.
    service: Arc<dyn PoolOperations + Send + Sync>,
}

impl PoolServer {
    pub fn new(service: Arc<dyn PoolOperations + Send + Sync>) -> Self {
        Self { service }
    }
    pub fn into_grpc_server(self) -> PoolGrpcServer<PoolServer> {
        PoolGrpcServer::new(self)
    }
}

impl Drop for PoolServer {
    fn drop(&mut self) {
        println!("DROPPING POOL SERVER")
    }
}

// Implementation of the RPC methods.
#[tonic::async_trait]
impl PoolGrpc for PoolServer {
    async fn destroy_pool(
        &self,
        request: Request<DestroyPoolRequest>,
    ) -> Result<tonic::Response<DestroyPoolReply>, tonic::Status> {
        let req = request.into_inner();
        // Dispatch the destroy call to the registered service.
        let service = self.service.clone();
        tokio::spawn(async move {
            match service.destroy(&req, None).await {
                Ok(()) => Ok(Response::new(DestroyPoolReply { error: None })),
                Err(e) => Ok(Response::new(DestroyPoolReply {
                    error: Some(e.into()),
                })),
            }
        })
        .await
        .unwrap_or_else(|_| {
            Ok(Response::new(DestroyPoolReply {
                error: Some(ReplyError::tonic_reply_error().into()),
            }))
        })
    }

    async fn create_pool(
        &self,
        request: Request<CreatePoolRequest>,
    ) -> Result<tonic::Response<pool_grpc::CreatePoolReply>, tonic::Status> {
        let req: CreatePoolRequest = request.into_inner();
        let service = self.service.clone();
        tokio::spawn(async move {
            match service.create(&req, None).await {
                Ok(pool) => Ok(Response::new(CreatePoolReply {
                    reply: Some(create_pool_reply::Reply::Pool(pool.into())),
                })),
                Err(err) => Ok(Response::new(CreatePoolReply {
                    reply: Some(create_pool_reply::Reply::Error(err.into())),
                })),
            }
        })
        .await
        .unwrap_or_else(|_| {
            Ok(Response::new(CreatePoolReply {
                reply: Some(create_pool_reply::Reply::Error(
                    ReplyError::tonic_reply_error().into(),
                )),
            }))
        })
    }

    async fn get_pools(
        &self,
        request: Request<GetPoolsRequest>,
    ) -> Result<tonic::Response<pool_grpc::GetPoolsReply>, tonic::Status> {
        let req: GetPoolsRequest = request.into_inner();
        let filter = if req.filter.is_none() {
            Filter::None
        } else {
            req.filter.unwrap().into()
        };
        let service = self.service.clone();
        tokio::spawn(async move {
            match service.get(filter, None).await {
                Ok(pools) => Ok(Response::new(GetPoolsReply {
                    reply: Some(get_pools_reply::Reply::Pools(pools.into())),
                })),
                Err(err) => Ok(Response::new(GetPoolsReply {
                    reply: Some(get_pools_reply::Reply::Error(err.into())),
                })),
            }
        })
        .await
        .unwrap_or_else(|_| {
            Ok(Response::new(GetPoolsReply {
                reply: Some(get_pools_reply::Reply::Error(
                    ReplyError::tonic_reply_error().into(),
                )),
            }))
        })
    }
}
