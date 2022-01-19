use crate::{
    common::{NodeFilter, NodePoolFilter, PoolFilter},
    get_core_ip,
    pool::traits::PoolOperations,
    pool_grpc::{
        create_pool_reply, get_pools_reply, get_pools_request, pool_grpc_client::PoolGrpcClient,
        CreatePoolRequest, DestroyPoolRequest, GetPoolsRequest,
    },
};
use std::time::Duration;
use tonic::transport::Uri;

use crate::pool::traits::{CreatePoolInfo, DestroyPoolInfo};
use common_lib::{
    mbus_api::{v0::Pools, ReplyError},
    types::v0::message_bus::{Filter, Pool},
};

// RPC Pool Client
pub struct PoolClient {
    client: PoolGrpcClient<tonic::transport::Channel>,
}

impl PoolClient {
    pub async fn init(addr: Option<Uri>) -> impl PoolOperations {
        let a = match addr {
            None => get_core_ip(),
            Some(addr) => addr,
        };
        let endpoint = tonic::transport::Endpoint::from(a)
            .connect_timeout(Duration::from_millis(250))
            .timeout(Duration::from_millis(250));
        let client = PoolGrpcClient::connect(endpoint).await.unwrap();
        Self { client }
    }
}

/// Implement pool operations supported by the Pool RPC client.
/// This converts the client side data into a RPC request.
#[tonic::async_trait]
impl PoolOperations for PoolClient {
    async fn create(
        &self,
        create_pool_req: &(dyn CreatePoolInfo + Sync + Send),
    ) -> Result<Pool, ReplyError> {
        let req: CreatePoolRequest = create_pool_req.into();
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
    async fn destroy(
        &self,
        destroy_pool_req: &(dyn DestroyPoolInfo + Sync + Send),
    ) -> Result<(), ReplyError> {
        let req: DestroyPoolRequest = destroy_pool_req.into();
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
