use crate::{
    common::{
        NodeFilter, NodePoolFilter, NodePoolReplicaFilter, NodeReplicaFilter, PoolFilter,
        PoolReplicaFilter, ReplicaFilter, VolumeFilter,
    },
    get_core_ip,
    replica::traits::ReplicaOperations,
    replica_grpc::{
        create_replica_reply, get_replicas_reply, get_replicas_request,
        replica_grpc_client::ReplicaGrpcClient, share_replica_reply, CreateReplicaRequest,
        DestroyReplicaRequest, GetReplicasRequest, ShareReplicaRequest, UnshareReplicaRequest,
    },
};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint, Uri};

use crate::{
    grpc_opts::{timeout_grpc, Context},
    replica::traits::{
        CreateReplicaInfo, DestroyReplicaInfo, ShareReplicaInfo, UnshareReplicaInfo,
    },
};
use common_lib::{
    mbus_api::{v0::Replicas, ReplyError, TimeoutOptions},
    types::v0::message_bus::{Filter, MessageIdVs, Replica},
    DEFAULT_REQ_TIMEOUT,
};

// RPC Replica Client
pub struct ReplicaClient {
    base_timeout: Duration,
    endpoint: Endpoint,
    //client: ReplicaGrpcClient<Channel>,
}

impl ReplicaClient {
    pub async fn init(addr: Option<Uri>, opts: Option<TimeoutOptions>) -> impl ReplicaOperations {
        let a = match addr {
            None => get_core_ip(),
            Some(addr) => addr,
        };
        let timeout = opts
            .clone()
            .map(|opt| opt.base_timeout())
            .unwrap_or_else(|| humantime::parse_duration(DEFAULT_REQ_TIMEOUT).unwrap());
        let endpoint = tonic::transport::Endpoint::from(a)
            .connect_timeout(Duration::from_millis(500))
            .timeout(timeout);
        ReplicaGrpcClient::connect(endpoint.clone()).await.unwrap();
        println!("{:?}", opts);
        Self {
            base_timeout: timeout,
            endpoint, //client,
        }
    }
    pub async fn reconnect(
        &self,
        ctx: Option<Context>,
        op_id: MessageIdVs,
    ) -> ReplicaGrpcClient<Channel> {
        println!("RECONNECTING WITH {} ms", self.base_timeout.as_secs());
        let ctx_timeout = ctx.map(|ctx| ctx.timeout).flatten();
        match ctx_timeout {
            None => {
                let endpoint = self
                    .endpoint
                    .clone()
                    .connect_timeout(Duration::from_millis(500))
                    .timeout(timeout_grpc(op_id, self.base_timeout));
                ReplicaGrpcClient::connect(endpoint.clone()).await.unwrap()
            }
            Some(timeout) => {
                let endpoint = self
                    .endpoint
                    .clone()
                    .connect_timeout(Duration::from_millis(500))
                    .timeout(timeout);
                ReplicaGrpcClient::connect(endpoint.clone()).await.unwrap()
            }
        }
    }
}

#[tonic::async_trait]
impl ReplicaOperations for ReplicaClient {
    async fn create(
        &self,
        req: &(dyn CreateReplicaInfo + Sync + Send),
        ctx: Option<Context>,
    ) -> Result<Replica, ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::CreateReplica).await;
        let req: CreateReplicaRequest = req.into();
        let response = client
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

    async fn get(&self, filter: Filter, ctx: Option<Context>) -> Result<Replicas, ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::GetReplicas).await;
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
        let response = client.clone().get_replicas(req).await.unwrap().into_inner();
        match response.reply.unwrap() {
            get_replicas_reply::Reply::Replicas(replicas) => Ok(replicas.into()),
            get_replicas_reply::Reply::Error(err) => Err(err.into()),
        }
    }

    async fn destroy(
        &self,
        req: &(dyn DestroyReplicaInfo + Sync + Send),
        ctx: Option<Context>,
    ) -> Result<(), ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::DestroyReplica).await;
        let req: DestroyReplicaRequest = req.into();
        let response = client
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

    async fn share(
        &self,
        req: &(dyn ShareReplicaInfo + Sync + Send),
        ctx: Option<Context>,
    ) -> Result<String, ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::ShareReplica).await;
        let req: ShareReplicaRequest = req.into();
        let response = client
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

    async fn unshare(
        &self,
        req: &(dyn UnshareReplicaInfo + Sync + Send),
        ctx: Option<Context>,
    ) -> Result<(), ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::UnshareReplica).await;
        let req: UnshareReplicaRequest = req.into();
        let response = client
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
