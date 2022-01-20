use crate::{
    replica::traits::ReplicaOperations,
    replica_grpc::{
        create_replica_reply, get_replicas_reply, replica_grpc_server::ReplicaGrpc,
        share_replica_reply, CreateReplicaReply, CreateReplicaRequest, DestroyReplicaReply,
        DestroyReplicaRequest, GetReplicasReply, GetReplicasRequest, ShareReplicaReply,
        ShareReplicaRequest, UnshareReplicaReply, UnshareReplicaRequest,
    },
};
use common_lib::types::v0::message_bus::Filter;
use std::sync::Arc;
use tonic::Response;

// RPC Replica Server
pub struct ReplicaServer {
    // Service which executes the operations.
    service: Arc<dyn ReplicaOperations + Send + Sync>,
}

impl Drop for ReplicaServer {
    fn drop(&mut self) {
        println!("DROPPING REPLICA SERVER")
    }
}

impl ReplicaServer {
    pub fn new(service: Arc<dyn ReplicaOperations + Send + Sync>) -> Self {
        Self { service }
    }
}

// Implementation of the RPC methods.
#[tonic::async_trait]
impl ReplicaGrpc for ReplicaServer {
    async fn create_replica(
        &self,
        request: tonic::Request<CreateReplicaRequest>,
    ) -> Result<tonic::Response<CreateReplicaReply>, tonic::Status> {
        let req = request.into_inner();
        match self.service.create(&req, None).await {
            Ok(replica) => Ok(Response::new(CreateReplicaReply {
                reply: Some(create_replica_reply::Reply::Replica(replica.into())),
            })),
            Err(err) => Ok(Response::new(CreateReplicaReply {
                reply: Some(create_replica_reply::Reply::Error(err.into())),
            })),
        }
    }
    async fn destroy_replica(
        &self,
        request: tonic::Request<DestroyReplicaRequest>,
    ) -> Result<tonic::Response<DestroyReplicaReply>, tonic::Status> {
        let req = request.into_inner();
        match self.service.destroy(&req, None).await {
            Ok(()) => Ok(Response::new(DestroyReplicaReply { error: None })),
            Err(e) => Ok(Response::new(DestroyReplicaReply {
                error: Some(e.into()),
            })),
        }
    }
    async fn get_replicas(
        &self,
        request: tonic::Request<GetReplicasRequest>,
    ) -> Result<tonic::Response<GetReplicasReply>, tonic::Status> {
        let req: GetReplicasRequest = request.into_inner();
        let filter: Filter = if req.filter.is_none() {
            Filter::None
        } else {
            req.filter.unwrap().into()
        };
        match self.service.get(filter, None).await {
            Ok(replicas) => Ok(Response::new(GetReplicasReply {
                reply: Some(get_replicas_reply::Reply::Replicas(replicas.into())),
            })),
            Err(err) => Ok(Response::new(GetReplicasReply {
                reply: Some(get_replicas_reply::Reply::Error(err.into())),
            })),
        }
    }
    async fn share_replica(
        &self,
        request: tonic::Request<ShareReplicaRequest>,
    ) -> Result<tonic::Response<ShareReplicaReply>, tonic::Status> {
        let req = request.into_inner();
        match self.service.share(&req, None).await {
            Ok(message) => Ok(Response::new(ShareReplicaReply {
                reply: Some(share_replica_reply::Reply::Response(message)),
            })),
            Err(err) => Ok(Response::new(ShareReplicaReply {
                reply: Some(share_replica_reply::Reply::Error(err.into())),
            })),
        }
    }
    async fn unshare_replica(
        &self,
        request: tonic::Request<UnshareReplicaRequest>,
    ) -> Result<tonic::Response<UnshareReplicaReply>, tonic::Status> {
        let req = request.into_inner();
        match self.service.unshare(&req, None).await {
            Ok(()) => Ok(Response::new(UnshareReplicaReply { error: None })),
            Err(e) => Ok(Response::new(UnshareReplicaReply {
                error: Some(e.into()),
            })),
        }
    }
}
