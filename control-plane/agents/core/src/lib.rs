use common::ServiceError;
use futures::future::join_all;
use grpc::{pool::server::PoolServer, replica::server::ReplicaServer};
use http::Uri;
use tracing::error;

pub struct Service {
    base_service: common::Service,
    tonic_grpc_server: tonic::transport::Server,
}

impl Service {
    pub fn new(base_service: common::Service) -> Self {
        Self {
            base_service,
            tonic_grpc_server: tonic::transport::Server::builder(),
        }
    }

    pub async fn run(mut self) {
        let grpc_addr = self.base_service.get_shared_state::<Uri>().clone();
        let pool_service = self.base_service.get_shared_state::<PoolServer>().clone();
        let replica_service = self
            .base_service
            .get_shared_state::<ReplicaServer>()
            .clone();

        let runnable = self
            .tonic_grpc_server
            .add_service(pool_service.into_grpc_server())
            .add_service(replica_service.into_grpc_server());

        let tonic_thread = tokio::spawn(async move {
            runnable
                .serve(grpc_addr.authority().unwrap().to_string().parse().unwrap())
                .await
                .map_err(|e| ServiceError::GrpcServer { source: e })
        });

        let mut threads = self.base_service.run().await;

        threads.push(tonic_thread);

        join_all(threads)
            .await
            .iter()
            .for_each(|result| match result {
                Err(error) => error!("Failed to wait for thread: {:?}", error),
                Ok(Err(error)) => {
                    error!("Error running channel thread: {:?}", error)
                }
                _ => {}
            });
    }
}
