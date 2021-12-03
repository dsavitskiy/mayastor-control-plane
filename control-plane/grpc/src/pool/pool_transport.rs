use crate::{
    pool::traits::PoolOperations,
    pool_grpc,
    pool_grpc::{
        create_pool_reply, get_pools_reply, get_pools_request,
        pool_grpc_client::PoolGrpcClient,
        pool_grpc_server::{PoolGrpc, PoolGrpcServer},
        CreatePoolReply, CreatePoolRequest, DestroyPoolReply, DestroyPoolRequest, GetPoolsReply,
        GetPoolsRequest, NodeFilter, NodePoolFilter, PoolFilter,
    },
};

const GRPC_SERVER: &str = "10.1.0.4:50051";
const HTTPS_SCHEME: &str = "https://";

use common_lib::{
    mbus_api::{v0::Pools, ReplyError},
    types::v0::message_bus::{Filter, Pool},
};
use std::{collections::HashMap, sync::Arc};
use tonic::{transport::Server, Request, Response};

// RPC Pool Client
pub struct PoolClient {
    client: PoolGrpcClient<tonic::transport::Channel>,
}

impl PoolClient {
    pub async fn init() -> impl PoolOperations {
        let client = PoolGrpcClient::connect(HTTPS_SCHEME.to_owned() + GRPC_SERVER)
            .await
            .unwrap();
        Self { client }
    }
}

/// Implement pool operations supported by the Pool RPC client.
/// This converts the client side data into a RPC request.
#[tonic::async_trait]
impl PoolOperations for PoolClient {
    async fn create(
        &self,
        id: String,
        node: String,
        disks: Vec<String>,
        labels: Option<HashMap<String, String>>,
    ) -> Result<Pool, ReplyError> {
        let req = CreatePoolRequest {
            pool_id: id,
            node_id: node,
            disks,
            labels: Some(crate::common::StringMapValue {
                value: labels.unwrap_or_default(),
            }),
        };
        let response = self
            .client
            .clone()
            .create_pool(req)
            .await
            .unwrap()
            .into_inner();
        match response.reply.unwrap() {
            create_pool_reply::Reply::Pool(pool) => Ok(pool.into()),
            create_pool_reply::Reply::Error(err) => Err(err.into()),
        }
    }

    /// Issue the pool destroy operation over RPC.
    async fn destroy(&self, node_id: String, pool_id: String) -> Result<(), ReplyError> {
        let req = DestroyPoolRequest { pool_id, node_id };
        let response = self
            .client
            .clone()
            .destroy_pool(req)
            .await
            .unwrap()
            .into_inner();
        match response.error {
            None => Ok(()),
            Some(err) => Err(err.into()),
        }
    }

    async fn get(&self, filter: Filter) -> Result<Pools, ReplyError> {
        let req: GetPoolsRequest = match filter {
            Filter::Node(id) => GetPoolsRequest {
                filter: Some(get_pools_request::Filter::Node(NodeFilter {
                    node_id: id.into(),
                })),
            },
            Filter::Pool(id) => GetPoolsRequest {
                filter: Some(get_pools_request::Filter::Pool(PoolFilter {
                    pool_id: id.into(),
                })),
            },
            Filter::NodePool(node_id, pool_id) => GetPoolsRequest {
                filter: Some(get_pools_request::Filter::NodePool(NodePoolFilter {
                    node_id: node_id.into(),
                    pool_id: pool_id.into(),
                })),
            },
            _ => GetPoolsRequest { filter: None },
        };
        let response = self
            .client
            .clone()
            .get_pools(req)
            .await
            .unwrap()
            .into_inner();
        match response.reply.unwrap() {
            get_pools_reply::Reply::Pools(pools) => Ok(pools.into()),
            get_pools_reply::Reply::Error(err) => Err(err.into()),
        }
    }
}

// RPC Pool Server
pub struct PoolServer {
    // Service which executes the operations.
    service: Arc<dyn PoolOperations + Send + Sync>,
}

impl Drop for PoolServer {
    fn drop(&mut self) {
        println!("DROPPING POOL SERVER")
    }
}

impl PoolServer {
    /// Create a new pool RPC server.
    /// This registers the service that should be called to satisfy the pool operations.
    pub async fn init(service: Arc<dyn PoolOperations + Send + Sync>) {
        println!("Starting Pool Server");
        tokio::spawn(async {
            let server = Self { service };
            let _ = server.start().await;
        });
        println!("Finished starting Pool Server");
    }

    /// Start the RPC server.
    async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = GRPC_SERVER.parse().unwrap();
        Server::builder()
            .add_service(PoolGrpcServer::new(self))
            .serve(addr)
            .await?;
        Ok(())
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
        let response = self.service.destroy(req.node_id, req.pool_id).await;
        match response {
            Ok(()) => Ok(Response::new(DestroyPoolReply { error: None })),
            Err(e) => Ok(Response::new(DestroyPoolReply {
                error: Some(e.into()),
            })),
        }
    }

    async fn create_pool(
        &self,
        request: Request<CreatePoolRequest>,
    ) -> Result<tonic::Response<pool_grpc::CreatePoolReply>, tonic::Status> {
        let req: CreatePoolRequest = request.into_inner();
        match self
            .service
            .create(
                req.pool_id,
                req.node_id,
                req.disks,
                Some(req.labels.unwrap_or_default().value),
            )
            .await
        {
            Ok(pool) => Ok(Response::new(CreatePoolReply {
                reply: Some(create_pool_reply::Reply::Pool(pool.into())),
            })),
            Err(err) => Ok(Response::new(CreatePoolReply {
                reply: Some(create_pool_reply::Reply::Error(err.into())),
            })),
        }
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
        match self.service.get(filter).await {
            Ok(pools) => Ok(Response::new(GetPoolsReply {
                reply: Some(get_pools_reply::Reply::Pools(pools.into())),
            })),
            Err(err) => Ok(Response::new(GetPoolsReply {
                reply: Some(get_pools_reply::Reply::Error(err.into())),
            })),
        }
    }
}
