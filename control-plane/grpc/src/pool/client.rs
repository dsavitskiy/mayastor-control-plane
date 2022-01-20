use crate::{
    common::{NodeFilter, NodePoolFilter, PoolFilter},
    get_core_ip,
    grpc_opts::{timeout_grpc, Context},
    pool::traits::{CreatePoolInfo, DestroyPoolInfo, PoolOperations},
    pool_grpc::{
        create_pool_reply, get_pools_reply, get_pools_request, pool_grpc_client::PoolGrpcClient,
        CreatePoolRequest, DestroyPoolRequest, GetPoolsRequest,
    },
};
use common_lib::{
    mbus_api::{v0::Pools, ReplyError, TimeoutOptions},
    types::v0::message_bus::{Filter, MessageIdVs, Pool},
};
use std::time::Duration;
use tonic::transport::{Channel, Endpoint, Uri};
use utils::DEFAULT_REQ_TIMEOUT;

// RPC Pool Client
pub struct PoolClient {
    base_timeout: Duration,
    endpoint: Endpoint,
    //client: PoolGrpcClient<Channel>,
}

impl PoolClient {
    pub async fn init<O: Into<Option<TimeoutOptions>>>(
        addr: Option<Uri>,
        opts: O,
    ) -> impl PoolOperations {
        let opts = opts.into();
        let a = match addr {
            None => get_core_ip(),
            Some(addr) => addr,
        };
        let timeout = opts
            .clone()
            .map(|opt| opt.base_timeout())
            .unwrap_or_else(|| humantime::parse_duration(DEFAULT_REQ_TIMEOUT).unwrap());
        let endpoint = tonic::transport::Endpoint::from(a)
            .connect_timeout(timeout)
            .timeout(timeout);
        println!("{:?}", opts);
        println!("Base: {:?}", timeout);
        Self {
            base_timeout: timeout,
            endpoint, //client,
        }
    }
    pub async fn reconnect(
        &self,
        ctx: Option<Context>,
        op_id: MessageIdVs,
    ) -> PoolGrpcClient<Channel> {
        println!("RECONNECTING WITH {} ms", self.base_timeout.as_millis());
        let ctx_timeout = ctx.map(|ctx| ctx.timeout_opts).flatten();
        match ctx_timeout {
            None => {
                let timeout = timeout_grpc(op_id, self.base_timeout);
                let endpoint = self
                    .endpoint
                    .clone()
                    .connect_timeout(timeout)
                    .timeout(timeout);
                PoolGrpcClient::connect(endpoint.clone()).await.unwrap()
            }
            Some(timeout) => {
                let timeout = timeout.base_timeout();
                let endpoint = self
                    .endpoint
                    .clone()
                    .connect_timeout(timeout)
                    .timeout(timeout);
                PoolGrpcClient::connect(endpoint.clone()).await.unwrap()
            }
        }
    }
}

/// Implement pool operations supported by the Pool RPC client.
/// This converts the client side data into a RPC request.
#[tonic::async_trait]
impl PoolOperations for PoolClient {
    async fn create(
        &self,
        create_pool_req: &(dyn CreatePoolInfo + Sync + Send),
        ctx: Option<Context>,
    ) -> Result<Pool, ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::CreatePool).await;
        let req: CreatePoolRequest = create_pool_req.into();
        let response = client.clone().create_pool(req).await.unwrap().into_inner();
        match response.reply.unwrap() {
            create_pool_reply::Reply::Pool(pool) => Ok(pool.into()),
            create_pool_reply::Reply::Error(err) => Err(err.into()),
        }
    }

    /// Issue the pool destroy operation over RPC.
    async fn destroy(
        &self,
        destroy_pool_req: &(dyn DestroyPoolInfo + Sync + Send),
        ctx: Option<Context>,
    ) -> Result<(), ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::DestroyPool).await;
        let req: DestroyPoolRequest = destroy_pool_req.into();
        let response = client.clone().destroy_pool(req).await.unwrap().into_inner();
        match response.error {
            None => Ok(()),
            Some(err) => Err(err.into()),
        }
    }

    async fn get(&self, filter: Filter, ctx: Option<Context>) -> Result<Pools, ReplyError> {
        let client = self.reconnect(ctx, MessageIdVs::GetPools).await;
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
        let response = client.clone().get_pools(req).await.unwrap().into_inner();
        match response.reply.unwrap() {
            get_pools_reply::Reply::Pools(pools) => Ok(pools.into()),
            get_pools_reply::Reply::Error(err) => Err(err.into()),
        }
    }
}
