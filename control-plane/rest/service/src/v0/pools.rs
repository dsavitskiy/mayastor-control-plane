use super::*;
use common_lib::types::v0::message_bus::{DestroyPool, Filter};
use grpc::pool::{client::PoolClient, traits::PoolOperations};
use mbus_api::{message_bus::v0::BusError, ReplyErrorKind, ResourceKind};

async fn destroy_pool(filter: Filter) -> Result<(), RestError<RestJsonError>> {
    let client = PoolClient::init(None, None).await;
    let destroy = match filter.clone() {
        Filter::NodePool(node_id, pool_id) => DestroyPool {
            node: node_id,
            id: pool_id,
        },
        Filter::Pool(pool_id) => {
            let node_id = match client.get(filter, None).await {
                Ok(pools) => pools.into_inner()[0].clone().node(),
                Err(error) => return Err(RestError::from(error)),
            };
            DestroyPool {
                node: node_id,
                id: pool_id,
            }
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Pool,
                source: "destroy_pool".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };
    client.destroy(&destroy, None).await?;
    Ok(())
}

#[async_trait::async_trait]
impl apis::actix_server::Pools for RestApi {
    async fn del_node_pool(
        Path((node_id, pool_id)): Path<(String, String)>,
    ) -> Result<(), RestError<RestJsonError>> {
        destroy_pool(Filter::NodePool(node_id.into(), pool_id.into())).await
    }

    async fn del_pool(Path(pool_id): Path<String>) -> Result<(), RestError<RestJsonError>> {
        destroy_pool(Filter::Pool(pool_id.into())).await
    }

    async fn get_node_pool(
        Path((node_id, pool_id)): Path<(String, String)>,
    ) -> Result<models::Pool, RestError<RestJsonError>> {
        let client = PoolClient::init(None, None).await;
        let pool = client
            .get(Filter::NodePool(node_id.into(), pool_id.into()), None)
            .await?;
        Ok(pool.into_inner()[0].clone().into())
    }

    async fn get_node_pools(
        Path(id): Path<String>,
    ) -> Result<Vec<models::Pool>, RestError<RestJsonError>> {
        let client = PoolClient::init(None, None).await;
        let pools = client.get(Filter::Node(id.into()), None).await?;
        Ok(pools.into_inner().into_iter().map(From::from).collect())
    }

    async fn get_pool(
        Path(pool_id): Path<String>,
    ) -> Result<models::Pool, RestError<RestJsonError>> {
        let client = PoolClient::init(None, None).await;
        let pool = client.get(Filter::Pool(pool_id.into()), None).await?;
        Ok(pool.into_inner()[0].clone().into())
    }

    async fn get_pools() -> Result<Vec<models::Pool>, RestError<RestJsonError>> {
        let client = PoolClient::init(None, None).await;
        let pools = client.get(Filter::None, None).await?;
        Ok(pools.into_inner().into_iter().map(From::from).collect())
    }

    async fn put_node_pool(
        Path((node_id, pool_id)): Path<(String, String)>,
        Body(create_pool_body): Body<models::CreatePoolBody>,
    ) -> Result<models::Pool, RestError<RestJsonError>> {
        let create =
            CreatePoolBody::from(create_pool_body).bus_request(node_id.into(), pool_id.into());
        let client = PoolClient::init(None, None).await;
        let pool = client.create(&create, None).await?;
        Ok(pool.into())
    }
}
