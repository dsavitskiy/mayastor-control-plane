use super::*;
use common_lib::{mbus_api::message_bus::v0::BusError, types::v0::openapi::apis::Uuid};
use grpc::{
    pool::{client::PoolClient, traits::PoolOperations},
    replica::{client::ReplicaClient, traits::ReplicaOperations},
};
use mbus_api::{ReplyErrorKind, ResourceKind};

async fn put_replica(
    filter: Filter,
    body: CreateReplicaBody,
) -> Result<models::Replica, RestError<RestJsonError>> {
    let pool_client = PoolClient::init(None, None).await;
    let create = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => {
            body.bus_request(node_id, pool_id, replica_id)
        }
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match pool_client.get(Filter::Pool(pool_id.clone()), None).await {
                Ok(pools) => pools.into_inner()[0].clone().node(),
                Err(error) => return Err(RestError::from(error)),
            };
            body.bus_request(node_id, pool_id, replica_id)
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "put_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };
    let client = ReplicaClient::init(None, None).await;
    let replica = client.create(&create, None).await?;
    Ok(replica.into())
}

async fn destroy_replica(filter: Filter) -> Result<(), RestError<RestJsonError>> {
    let client = ReplicaClient::init(None, None).await;
    let destroy = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => DestroyReplica {
            node: node_id,
            pool: pool_id,
            name: None,
            uuid: replica_id,
            ..Default::default()
        },
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match client.get(filter, None).await {
                Ok(replicas) => replicas.into_inner()[0].clone().node,
                Err(error) => return Err(RestError::from(error)),
            };

            DestroyReplica {
                node: node_id,
                pool: pool_id,
                name: None,
                uuid: replica_id,
                ..Default::default()
            }
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "destroy_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };
    client.destroy(&destroy, None).await?;
    Ok(())
}

async fn share_replica(
    filter: Filter,
    protocol: ReplicaShareProtocol,
) -> Result<String, RestError<RestJsonError>> {
    let client = ReplicaClient::init(None, None).await;
    let share = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => ShareReplica {
            node: node_id,
            pool: pool_id,
            name: None,
            uuid: replica_id,
            protocol,
        },
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match client.get(filter, None).await {
                Ok(replicas) => replicas.into_inner()[0].clone().node,
                Err(error) => return Err(RestError::from(error)),
            };

            ShareReplica {
                node: node_id,
                pool: pool_id,
                name: None,
                uuid: replica_id,
                protocol,
            }
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "share_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };
    let share_uri = client.share(&share, None).await?;
    Ok(share_uri)
}

async fn unshare_replica(filter: Filter) -> Result<(), RestError<RestJsonError>> {
    let client = ReplicaClient::init(None, None).await;
    let unshare = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => UnshareReplica {
            node: node_id,
            pool: pool_id,
            name: None,
            uuid: replica_id,
        },
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match client.get(filter, None).await {
                Ok(replicas) => replicas.into_inner()[0].clone().node,
                Err(error) => return Err(RestError::from(error)),
            };

            UnshareReplica {
                node: node_id,
                pool: pool_id,
                name: None,
                uuid: replica_id,
            }
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "unshare_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };

    client.unshare(&unshare, None).await?;
    Ok(())
}

#[async_trait::async_trait]
impl apis::actix_server::Replicas for RestApi {
    async fn del_node_pool_replica(
        Path((node_id, pool_id, replica_id)): Path<(String, String, Uuid)>,
    ) -> Result<(), RestError<RestJsonError>> {
        destroy_replica(Filter::NodePoolReplica(
            node_id.into(),
            pool_id.into(),
            replica_id.into(),
        ))
        .await
    }

    async fn del_node_pool_replica_share(
        Path((node_id, pool_id, replica_id)): Path<(String, String, Uuid)>,
    ) -> Result<(), RestError<RestJsonError>> {
        unshare_replica(Filter::NodePoolReplica(
            node_id.into(),
            pool_id.into(),
            replica_id.into(),
        ))
        .await
    }

    async fn del_pool_replica(
        Path((pool_id, replica_id)): Path<(String, Uuid)>,
    ) -> Result<(), RestError<RestJsonError>> {
        destroy_replica(Filter::PoolReplica(pool_id.into(), replica_id.into())).await
    }

    async fn del_pool_replica_share(
        Path((pool_id, replica_id)): Path<(String, Uuid)>,
    ) -> Result<(), RestError<RestJsonError>> {
        unshare_replica(Filter::PoolReplica(pool_id.into(), replica_id.into())).await
    }

    async fn get_node_pool_replica(
        Path((node_id, pool_id, replica_id)): Path<(String, String, Uuid)>,
    ) -> Result<models::Replica, RestError<RestJsonError>> {
        let client = ReplicaClient::init(None, None).await;
        let replicas = client
            .get(
                Filter::NodePoolReplica(node_id.into(), pool_id.into(), replica_id.into()),
                None,
            )
            .await?;
        Ok(replicas.into_inner()[0].clone().into())
    }

    async fn get_node_pool_replicas(
        Path((node_id, pool_id)): Path<(String, String)>,
    ) -> Result<Vec<models::Replica>, RestError<RestJsonError>> {
        let client = ReplicaClient::init(None, None).await;
        let replicas = client
            .get(Filter::NodePool(node_id.into(), pool_id.into()), None)
            .await?;
        Ok(replicas.into_inner().into_iter().map(From::from).collect())
    }

    async fn get_node_replicas(
        Path(id): Path<String>,
    ) -> Result<Vec<models::Replica>, RestError<RestJsonError>> {
        let client = ReplicaClient::init(None, None).await;
        let replicas = client.get(Filter::Node(id.into()), None).await?;
        Ok(replicas.into_inner().into_iter().map(From::from).collect())
    }

    async fn get_replica(
        Path(id): Path<Uuid>,
    ) -> Result<models::Replica, RestError<RestJsonError>> {
        let client = ReplicaClient::init(None, None).await;
        let replicas = client.get(Filter::Replica(id.into()), None).await?;
        Ok(replicas.into_inner()[0].clone().into())
    }

    async fn get_replicas() -> Result<Vec<models::Replica>, RestError<RestJsonError>> {
        let client = ReplicaClient::init(None, None).await;
        let replicas = client.get(Filter::None, None).await?;
        Ok(replicas.into_inner().into_iter().map(From::from).collect())
    }

    async fn put_node_pool_replica(
        Path((node_id, pool_id, replica_id)): Path<(String, String, Uuid)>,
        Body(create_replica_body): Body<models::CreateReplicaBody>,
    ) -> Result<models::Replica, RestError<RestJsonError>> {
        put_replica(
            Filter::NodePoolReplica(node_id.into(), pool_id.into(), replica_id.into()),
            CreateReplicaBody::from(create_replica_body),
        )
        .await
    }

    async fn put_node_pool_replica_share(
        Path((node_id, pool_id, replica_id)): Path<(String, String, Uuid)>,
    ) -> Result<String, RestError<RestJsonError>> {
        share_replica(
            Filter::NodePoolReplica(node_id.into(), pool_id.into(), replica_id.into()),
            ReplicaShareProtocol::Nvmf,
        )
        .await
    }

    async fn put_pool_replica(
        Path((pool_id, replica_id)): Path<(String, Uuid)>,
        Body(create_replica_body): Body<models::CreateReplicaBody>,
    ) -> Result<models::Replica, RestError<RestJsonError>> {
        put_replica(
            Filter::PoolReplica(pool_id.into(), replica_id.into()),
            CreateReplicaBody::from(create_replica_body),
        )
        .await
    }

    async fn put_pool_replica_share(
        Path((pool_id, replica_id)): Path<(String, Uuid)>,
    ) -> Result<String, RestError<RestJsonError>> {
        share_replica(
            Filter::PoolReplica(pool_id.into(), replica_id.into()),
            ReplicaShareProtocol::Nvmf,
        )
        .await
    }
}
