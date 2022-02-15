use crate::operations::registration::traits::{
    DeregisterRequest, RegisterRequest, RegistrationOperations,
};

use rpc::{
    registration,
    registration::{
        registration_server, registration_server::Registration, ProbeRequest, ProbeResponse,
    },
};
use std::sync::Arc;
use tonic::{Code, Response};

/// RPC Registration Server
#[derive(Clone)]
pub struct RegistrationServer {
    /// Service which executes the operations.
    service: Arc<dyn RegistrationOperations>,
}

impl RegistrationServer {
    /// returns a new Registration server with the service implementing Registration operations
    pub fn new(service: Arc<dyn RegistrationOperations>) -> Self {
        Self { service }
    }
    /// coverts the Registration server to its corresponding grpc server type
    pub fn into_grpc_server(self) -> registration_server::RegistrationServer<RegistrationServer> {
        registration_server::RegistrationServer::new(self)
    }
}

/// Implementation of the RPC methods.
#[tonic::async_trait]
impl Registration for RegistrationServer {
    async fn register(
        &self,
        request: tonic::Request<registration::RegisterRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let req = RegisterRequest(request.into_inner()).into();
        match self.service.register(req).await {
            Ok(()) => Ok(Response::new(())),
            Err(_) => Err(tonic::Status::new(Code::Aborted, "".to_string())),
        }
    }
    async fn deregister(
        &self,
        request: tonic::Request<registration::DeregisterRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let req = DeregisterRequest(request.into_inner()).into();
        match self.service.deregister(req).await {
            Ok(()) => Ok(Response::new(())),
            Err(_) => Err(tonic::Status::new(Code::Aborted, "".to_string())),
        }
    }
    async fn probe(
        &self,
        _request: tonic::Request<ProbeRequest>,
    ) -> Result<tonic::Response<ProbeResponse>, tonic::Status> {
        match self.service.probe(None).await {
            Ok(resp) => Ok(Response::new(ProbeResponse { ready: resp })),
            Err(_) => Ok(Response::new(ProbeResponse { ready: false })),
        }
    }
}
