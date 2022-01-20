use crate::{
    pool::traits::PoolOperations,
    pool_grpc,
    pool_grpc::{
        create_pool_reply, get_pools_reply, pool_grpc_server::PoolGrpc, CreatePoolReply,
        CreatePoolRequest, DestroyPoolReply, DestroyPoolRequest, GetPoolsReply, GetPoolsRequest,
    },
};
use std::sync::Arc;
use tonic::{Request, Response};

use common_lib::types::v0::message_bus::Filter;
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
    pub fn new(service: Arc<dyn PoolOperations + Send + Sync>) -> Self {
        Self { service }
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
        let response = self.service.destroy(&req, None).await;
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
        match self.service.create(&req, None).await {
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
        match self.service.get(filter, None).await {
            Ok(pools) => Ok(Response::new(GetPoolsReply {
                reply: Some(get_pools_reply::Reply::Pools(pools.into())),
            })),
            Err(err) => Ok(Response::new(GetPoolsReply {
                reply: Some(get_pools_reply::Reply::Error(err.into())),
            })),
        }
    }
}
