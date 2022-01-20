use crate::{
    pool::{server::PoolServer, traits::PoolOperations},
    pool_grpc::pool_grpc_server::PoolGrpcServer,
    replica::{server::ReplicaServer, traits::ReplicaOperations},
    replica_grpc::replica_grpc_server::ReplicaGrpcServer,
};
use std::sync::Arc;
use tonic::transport::{Server, Uri};

// RPC Pool Server
pub struct CoreServer {}

impl Drop for CoreServer {
    fn drop(&mut self) {
        println!("DROPPING CORE SERVER")
    }
}

impl CoreServer {
    /// Create a new pool RPC server.
    /// This registers the service that should be called to satisfy the pool operations.
    pub async fn init(
        pool_service: Arc<dyn PoolOperations + Send + Sync>,
        replica_service: Arc<dyn ReplicaOperations + Send + Sync>,
        addr: Uri,
    ) {
        println!("Starting Core Server");
        tokio::spawn(async {
            let server = Self {};
            let _ = server.start(pool_service, replica_service, addr).await;
        });
        println!("Finished starting Core Server");
    }

    /// Start the RPC server.
    async fn start(
        self,
        pool_service: Arc<dyn PoolOperations + Send + Sync>,
        replica_service: Arc<dyn ReplicaOperations + Send + Sync>,
        addr: Uri,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.authority().unwrap().to_string().parse().unwrap();
        println!("{:?}", addr);
        Server::builder()
            .add_service(PoolGrpcServer::new(PoolServer::new(pool_service)))
            .add_service(ReplicaGrpcServer::new(ReplicaServer::new(replica_service)))
            .serve(addr)
            .await?;
        tracing::error!("Stopped");
        Ok(())
    }
}
