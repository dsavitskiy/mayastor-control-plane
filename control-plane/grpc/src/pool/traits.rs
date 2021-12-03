use crate::{pool_grpc, pool_grpc::get_pools_request};
use common_lib::{
    mbus_api::{v0::Pools, ReplyError},
    types::v0::{
        message_bus::{Filter, Pool, PoolState, PoolStatus},
        store::pool::{PoolSpec, PoolSpecStatus},
    },
};
use std::collections::HashMap;

/// Trait implemented by services which support pool operations.
/// This trait can only be implemented on types which support the PoolInfo trait.
#[tonic::async_trait]
pub trait PoolOperations {
    async fn create(
        &self,
        id: String,
        node: String,
        disks: Vec<String>,
        labels: Option<HashMap<String, String>>,
    ) -> Result<Pool, ReplyError>;
    async fn destroy(&self, node_id: String, pool_id: String) -> Result<(), ReplyError>;
    async fn get(&self, filter: Filter) -> Result<Pools, ReplyError>;
}

impl From<pool_grpc::Pool> for Pool {
    fn from(pool: pool_grpc::Pool) -> Self {
        let pool_state = match pool.state {
            None => None,
            Some(pool_state) => Some(PoolState {
                node: pool_state.clone().node_id.into(),
                id: pool_state.clone().pool_id.into(),
                disks: pool_state.disks_uri.iter().map(|i| i.into()).collect(),
                status: pool_state.status.into(),
                capacity: pool_state.capacity,
                used: pool_state.used,
            }),
        };
        let pool_spec = match pool.definition {
            None => None,
            Some(pool_definition) => {
                let pool_spec = pool_definition.clone().spec.unwrap();
                let pool_meta = pool_definition.metadata.unwrap();
                let pool_spec_status = match pool_meta.status {
                    1 => match pool_state.clone() {
                        None => PoolSpecStatus::Created(PoolStatus::Unknown),
                        Some(state) => PoolSpecStatus::Created(state.status),
                    },
                    _ => pool_meta.status.into(),
                };
                Some(PoolSpec {
                    node: pool_spec.clone().node_id.into(),
                    id: pool_spec.clone().pool_id.into(),
                    disks: pool_spec.disks.iter().map(|i| i.into()).collect(),
                    status: pool_spec_status,
                    labels: pool_spec.labels.unwrap_or_default().value.into(),
                    sequencer: Default::default(),
                    operation: None,
                })
            }
        };
        Pool::try_new(pool_spec, pool_state).unwrap()
    }
}

impl From<Pool> for pool_grpc::Pool {
    fn from(pool: Pool) -> Self {
        let pool_definition = match pool.spec() {
            None => None,
            Some(pool_spec) => Some(pool_grpc::PoolDefinition {
                spec: Some(pool_grpc::PoolSpec {
                    node_id: pool_spec.node.to_string(),
                    pool_id: pool_spec.id.to_string(),
                    disks: pool_spec.disks.iter().map(|i| i.to_string()).collect(),
                    labels: Some(crate::common::StringMapValue {
                        value: pool_spec.labels.unwrap_or_default(),
                    }),
                }),
                metadata: Some(pool_grpc::Metadata {
                    uuid: None,
                    status: pool_spec.status.into(),
                }),
            }),
        };
        let pool_state = match pool.state() {
            None => None,
            Some(pool_state) => Some(pool_grpc::PoolState {
                node_id: pool_state.node.to_string(),
                pool_id: pool_state.id.to_string(),
                disks_uri: pool_state.disks.iter().map(|i| i.to_string()).collect(),
                status: pool_state.status.into(),
                capacity: pool_state.capacity,
                used: pool_state.used,
            }),
        };
        pool_grpc::Pool {
            definition: pool_definition,
            state: pool_state,
        }
    }
}

impl From<pool_grpc::Pools> for Pools {
    fn from(pools: pool_grpc::Pools) -> Self {
        Pools(pools.pools.iter().map(|pool| pool.clone().into()).collect())
    }
}

impl From<Pools> for pool_grpc::Pools {
    fn from(pools: Pools) -> Self {
        pool_grpc::Pools {
            pools: pools
                .into_inner()
                .iter()
                .map(|pool| pool.clone().into())
                .collect(),
        }
    }
}

impl From<i32> for crate::common::SpecStatus {
    fn from(i: i32) -> Self {
        match i {
            0 => crate::common::SpecStatus::Creating,
            1 => crate::common::SpecStatus::Created,
            2 => crate::common::SpecStatus::Deleting,
            3 => crate::common::SpecStatus::Deleted,
            _ => crate::common::SpecStatus::Creating,
        }
    }
}

impl From<i32> for crate::pool_grpc::PoolStatus {
    fn from(i: i32) -> Self {
        match i {
            0 => crate::pool_grpc::PoolStatus::Unknown,
            1 => crate::pool_grpc::PoolStatus::Online,
            2 => crate::pool_grpc::PoolStatus::Degraded,
            3 => crate::pool_grpc::PoolStatus::Faulted,
            _ => crate::pool_grpc::PoolStatus::Unknown,
        }
    }
}

impl From<get_pools_request::Filter> for Filter {
    fn from(filter: get_pools_request::Filter) -> Self {
        match filter {
            get_pools_request::Filter::Node(node_filter) => {
                Filter::Node(node_filter.node_id.into())
            }
            get_pools_request::Filter::NodePool(node_pool_filter) => Filter::NodePool(
                node_pool_filter.node_id.into(),
                node_pool_filter.pool_id.into(),
            ),
            get_pools_request::Filter::Pool(pool_filter) => {
                Filter::Pool(pool_filter.pool_id.into())
            }
        }
    }
}

impl From<ReplyError> for crate::common::ReplyError {
    fn from(err: ReplyError) -> Self {
        crate::common::ReplyError {
            kind: err.clone().kind.into(),
            resource: err.clone().kind.into(),
            source: err.clone().source,
            extra: err.extra,
        }
    }
}

impl From<crate::common::ReplyError> for ReplyError {
    fn from(err: crate::common::ReplyError) -> Self {
        ReplyError {
            kind: err.clone().kind.into(),
            resource: err.clone().kind.into(),
            source: err.clone().source,
            extra: err.extra,
        }
    }
}
