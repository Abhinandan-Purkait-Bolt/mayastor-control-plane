use crate::{
    pool::{client::PoolClient, traits::PoolOperations},
    replica::{client::ReplicaClient, traits::ReplicaOperations},
};
use common_lib::mbus_api::TimeoutOptions;
use tonic::transport::Uri;

pub struct CoreClient {
    pool_client: PoolClient,
    replica_client: ReplicaClient,
}

impl CoreClient {
    pub async fn init<O: Into<Option<TimeoutOptions>>>(addr: Uri, opts: O) -> Self {
        let opts_clone = opts.into();
        let pool_client = PoolClient::init(Some(addr.clone()), opts_clone.clone()).await;
        let replica_client = ReplicaClient::init(Some(addr), opts_clone).await;
        Self {
            pool_client,
            replica_client,
        }
    }
    pub fn pool_client(&self) -> impl PoolOperations {
        self.pool_client.clone()
    }
    pub fn replica_client(&self) -> impl ReplicaOperations {
        self.replica_client.clone()
    }
}
