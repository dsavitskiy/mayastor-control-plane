use crate::{
    common::{
        NodeFilter, NodePoolFilter, NodePoolReplicaFilter, NodeReplicaFilter, PoolFilter,
        PoolReplicaFilter, ReplicaFilter, VolumeFilter,
    },
    replica::traits::ReplicaOperations,
    replica_grpc::{
        create_replica_reply, get_replicas_reply, get_replicas_request,
        replica_grpc_client::ReplicaGrpcClient, share_replica_reply, CreateReplicaRequest,
        DestroyReplicaRequest, GetReplicasRequest, ShareReplicaRequest, UnshareReplicaRequest,
    },
};

use common_lib::{
    mbus_api::{v0::Replicas, ReplyError},
    types::v0::message_bus::{
        CreateReplica, DestroyReplica, Filter, Replica, ShareReplica, UnshareReplica,
    },
};
const GRPC_SERVER: &str = "10.1.0.4:50051";
const HTTPS_SCHEME: &str = "https://";

// RPC Replica Client
pub struct ReplicaClient {
    client: ReplicaGrpcClient<tonic::transport::Channel>,
}

impl ReplicaClient {
    pub async fn init() -> impl ReplicaOperations {
        let client = ReplicaGrpcClient::connect(HTTPS_SCHEME.to_owned() + GRPC_SERVER)
            .await
            .unwrap();
        Self { client }
    }
}

#[tonic::async_trait]
impl ReplicaOperations for ReplicaClient {
    async fn create(&self, req: CreateReplica) -> Result<Replica, ReplyError> {
        let req: CreateReplicaRequest = req.into();
        let response = self
            .client
            .clone()
            .create_replica(req)
            .await
            .unwrap()
            .into_inner();
        match response.reply.unwrap() {
            create_replica_reply::Reply::Replica(replica) => Ok(replica.into()),
            create_replica_reply::Reply::Error(err) => Err(err.into()),
        }
    }

    async fn get(&self, filter: Filter) -> Result<Replicas, ReplyError> {
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
        let response = self
            .client
            .clone()
            .get_replicas(req)
            .await
            .unwrap()
            .into_inner();
        match response.reply.unwrap() {
            get_replicas_reply::Reply::Replicas(replicas) => Ok(replicas.into()),
            get_replicas_reply::Reply::Error(err) => Err(err.into()),
        }
    }

    async fn destroy(&self, req: DestroyReplica) -> Result<(), ReplyError> {
        let req: DestroyReplicaRequest = req.into();
        let response = self
            .client
            .clone()
            .destroy_replica(req)
            .await
            .unwrap()
            .into_inner();
        match response.error {
            None => Ok(()),
            Some(err) => Err(err.into()),
        }
    }

    async fn share(&self, req: ShareReplica) -> Result<String, ReplyError> {
        let req: ShareReplicaRequest = req.into();
        let response = self
            .client
            .clone()
            .share_replica(req)
            .await
            .unwrap()
            .into_inner();
        match response.reply.unwrap() {
            share_replica_reply::Reply::Response(message) => Ok(message),
            share_replica_reply::Reply::Error(err) => Err(err.into()),
        }
    }

    async fn unshare(&self, req: UnshareReplica) -> Result<(), ReplyError> {
        let req: UnshareReplicaRequest = req.into();
        let response = self
            .client
            .clone()
            .unshare_replica(req)
            .await
            .unwrap()
            .into_inner();
        match response.error {
            None => Ok(()),
            Some(err) => Err(err.into()),
        }
    }
}
