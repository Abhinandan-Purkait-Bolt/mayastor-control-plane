use crate::grpc_opts::Context;
use common_lib::{
    mbus_api::ReplyError,
    types::v0::message_bus::{Deregister, Register},
};

/// RegisterRequest type to encapsulate rpc RegisterRequest type
pub struct RegisterRequest(pub rpc::registration::RegisterRequest);
/// DeregisterRequest type to encapsulate rpc DeregisterRequest type
pub struct DeregisterRequest(pub rpc::registration::DeregisterRequest);

/// Operations to be supportes by the Registration Service
#[tonic::async_trait]
pub trait RegistrationOperations: Send + Sync {
    /// Register a dataplane node to controlplane
    async fn register(&self, req: Register) -> Result<(), ReplyError>;
    /// Deregister a dataplane node to controlplane
    async fn deregister(&self, req: Deregister) -> Result<(), ReplyError>;
    /// Liveness probe for the registration service
    async fn probe(&self, ctx: Option<Context>) -> Result<bool, ReplyError>;
}

impl From<Register> for RegisterRequest {
    fn from(register: Register) -> Self {
        Self(rpc::registration::RegisterRequest {
            id: register.id.to_string(),
            grpc_endpoint: register.grpc_endpoint,
        })
    }
}

impl From<Deregister> for DeregisterRequest {
    fn from(deregister: Deregister) -> Self {
        Self(rpc::registration::DeregisterRequest {
            id: deregister.id.to_string(),
        })
    }
}

impl From<RegisterRequest> for Register {
    fn from(register: RegisterRequest) -> Self {
        Self {
            id: register.0.id.into(),
            grpc_endpoint: register.0.grpc_endpoint,
        }
    }
}

impl From<DeregisterRequest> for Deregister {
    fn from(deregister: DeregisterRequest) -> Self {
        Self {
            id: deregister.0.id.into(),
        }
    }
}
