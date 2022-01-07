use crate::{
    common::{NodeFilter, NodePoolFilter, PoolFilter},
    pool::traits::PoolOperations,
    pool_grpc::{
        create_pool_reply, get_pools_reply, get_pools_request, pool_grpc_client::PoolGrpcClient,
        CreatePoolRequest, DestroyPoolRequest, GetPoolsRequest,
    },
};
use std::collections::HashMap;

use common_lib::{
    mbus_api::{v0::Pools, ReplyError},
    types::v0::message_bus::{Filter, Pool},
};

const GRPC_SERVER: &str = "10.1.0.4:50051";
const HTTPS_SCHEME: &str = "https://";

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
