use crate::{
    replica_grpc,
    replica_grpc::{
        get_replicas_request, CreateReplicaRequest, DestroyReplicaRequest, ShareReplicaRequest,
        UnshareReplicaRequest,
    },
};
use common_lib::{
    mbus_api::{v0::Replicas, ReplyError},
    types::v0::message_bus::{
        CreateReplica, DestroyReplica, Filter, NexusId, Protocol, Replica, ReplicaId,
        ReplicaOwners, ReplicaShareProtocol, ReplicaStatus, ShareReplica, UnshareReplica, VolumeId,
    },
};
use std::convert::TryFrom;

#[tonic::async_trait]
pub trait ReplicaOperations {
    async fn create(
        &self,
        req: &(dyn CreateReplicaInfo + Sync + Send),
    ) -> Result<Replica, ReplyError>;
    async fn get(&self, filter: Filter) -> Result<Replicas, ReplyError>;
    async fn destroy(&self, req: &(dyn DestroyReplicaInfo + Sync + Send))
        -> Result<(), ReplyError>;
    async fn share(&self, req: &(dyn ShareReplicaInfo + Sync + Send))
        -> Result<String, ReplyError>;
    async fn unshare(&self, req: &(dyn UnshareReplicaInfo + Sync + Send))
        -> Result<(), ReplyError>;
}

impl From<Replica> for replica_grpc::Replica {
    fn from(replica: Replica) -> Self {
        replica_grpc::Replica {
            node_id: replica.node.into(),
            name: replica.name.into(),
            replica_id: Some(replica.uuid.into()),
            pool_id: replica.pool.into(),
            thin: replica.thin,
            size: replica.size,
            share: replica.share.into(),
            uri: replica.uri,
            status: replica.status.into(),
        }
    }
}

impl From<replica_grpc::Replica> for Replica {
    fn from(replica: replica_grpc::Replica) -> Self {
        Replica {
            node: replica.node_id.into(),
            name: replica.name.into(),
            uuid: ReplicaId::try_from(replica.replica_id.unwrap()).unwrap(),
            pool: replica.pool_id.into(),
            thin: replica.thin,
            size: replica.size,
            share: replica.share.into(),
            uri: replica.uri,
            status: replica.status.into(),
        }
    }
}

impl From<replica_grpc::ReplicaStatus> for ReplicaStatus {
    fn from(status: replica_grpc::ReplicaStatus) -> Self {
        match status {
            replica_grpc::ReplicaStatus::Unknown => ReplicaStatus::Unknown,
            replica_grpc::ReplicaStatus::Online => ReplicaStatus::Online,
            replica_grpc::ReplicaStatus::Degraded => ReplicaStatus::Degraded,
            replica_grpc::ReplicaStatus::Faulted => ReplicaStatus::Faulted,
        }
    }
}

impl From<get_replicas_request::Filter> for Filter {
    fn from(filter: get_replicas_request::Filter) -> Self {
        match filter {
            get_replicas_request::Filter::Node(node_filter) => {
                Filter::Node(node_filter.node_id.into())
            }
            get_replicas_request::Filter::NodePool(node_pool_filter) => Filter::NodePool(
                node_pool_filter.node_id.into(),
                node_pool_filter.pool_id.into(),
            ),
            get_replicas_request::Filter::Pool(pool_filter) => {
                Filter::Pool(pool_filter.pool_id.into())
            }
            get_replicas_request::Filter::NodePoolReplica(node_pool_replica_filter) => {
                Filter::NodePoolReplica(
                    node_pool_replica_filter.node_id.into(),
                    node_pool_replica_filter.pool_id.into(),
                    ReplicaId::try_from(node_pool_replica_filter.replica_id).unwrap(),
                )
            }
            get_replicas_request::Filter::NodeReplica(node_replica_filter) => Filter::NodeReplica(
                node_replica_filter.node_id.into(),
                ReplicaId::try_from(node_replica_filter.replica_id).unwrap(),
            ),
            get_replicas_request::Filter::PoolReplica(pool_replica_filter) => Filter::PoolReplica(
                pool_replica_filter.pool_id.into(),
                ReplicaId::try_from(pool_replica_filter.replica_id).unwrap(),
            ),
            get_replicas_request::Filter::Replica(replica_filter) => {
                Filter::Replica(ReplicaId::try_from(replica_filter.replica_id).unwrap())
            }
            get_replicas_request::Filter::Volume(volume_filter) => {
                Filter::Volume(VolumeId::try_from(volume_filter.volume_id).unwrap())
            }
        }
    }
}

impl From<replica_grpc::Replicas> for Replicas {
    fn from(replicas: replica_grpc::Replicas) -> Self {
        Replicas(
            replicas
                .replicas
                .iter()
                .map(|replica| replica.clone().into())
                .collect(),
        )
    }
}

impl From<Replicas> for replica_grpc::Replicas {
    fn from(replicas: Replicas) -> Self {
        replica_grpc::Replicas {
            replicas: replicas
                .into_inner()
                .iter()
                .map(|replicas| replicas.clone().into())
                .collect(),
        }
    }
}

pub trait CreateReplicaInfo {
    fn node(&self) -> String;
    fn name(&self) -> Option<String>;
    fn uuid(&self) -> String;
    fn pool(&self) -> String;
    fn size(&self) -> u64;
    fn thin(&self) -> bool;
    fn share(&self) -> String;
    fn managed(&self) -> bool;
    fn owners(&self) -> ReplicaOwners;
}

impl CreateReplicaInfo for CreateReplica {
    fn node(&self) -> String {
        self.node.to_string()
    }

    fn name(&self) -> Option<String> {
        self.name.clone().map(|name| name.to_string())
    }

    fn uuid(&self) -> String {
        self.uuid.to_string()
    }

    fn pool(&self) -> String {
        self.pool.to_string()
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn thin(&self) -> bool {
        self.thin
    }

    fn share(&self) -> String {
        self.share.to_string()
    }

    fn managed(&self) -> bool {
        self.managed
    }

    fn owners(&self) -> ReplicaOwners {
        self.owners.clone()
    }
}

impl CreateReplicaInfo for CreateReplicaRequest {
    fn node(&self) -> String {
        self.node_id.clone()
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn uuid(&self) -> String {
        self.replica_id.clone().unwrap()
    }

    fn pool(&self) -> String {
        self.pool_id.clone()
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn thin(&self) -> bool {
        self.thin
    }

    fn share(&self) -> String {
        let protocol: Protocol = self.share.into();
        protocol.to_string()
    }

    fn managed(&self) -> bool {
        self.managed
    }

    fn owners(&self) -> ReplicaOwners {
        ReplicaOwners::new(
            self.owners
                .clone()
                .unwrap()
                .volume
                .map(|id| VolumeId::try_from(id).unwrap()),
            self.owners
                .clone()
                .unwrap()
                .nexuses
                .iter()
                .map(|id| NexusId::try_from(id.clone()).unwrap())
                .collect(),
        )
    }
}

pub trait DestroyReplicaInfo {
    fn node(&self) -> String;
    fn pool(&self) -> String;
    fn name(&self) -> Option<String>;
    fn uuid(&self) -> String;
    fn disowners(&self) -> ReplicaOwners;
}

impl DestroyReplicaInfo for DestroyReplica {
    fn node(&self) -> String {
        self.node.to_string()
    }

    fn pool(&self) -> String {
        self.pool.to_string()
    }

    fn name(&self) -> Option<String> {
        self.name.clone().map(|name| name.to_string())
    }

    fn uuid(&self) -> String {
        self.uuid.to_string()
    }

    fn disowners(&self) -> ReplicaOwners {
        self.disowners.clone()
    }
}

impl DestroyReplicaInfo for DestroyReplicaRequest {
    fn node(&self) -> String {
        self.node_id.clone()
    }

    fn pool(&self) -> String {
        self.pool_id.clone()
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn uuid(&self) -> String {
        self.replica_id.clone().unwrap()
    }

    fn disowners(&self) -> ReplicaOwners {
        ReplicaOwners::new(None, vec![])
    }
}

pub trait ShareReplicaInfo {
    fn node(&self) -> String;
    fn pool(&self) -> String;
    fn name(&self) -> Option<String>;
    fn uuid(&self) -> String;
    fn protocol(&self) -> String;
}

impl ShareReplicaInfo for ShareReplica {
    fn node(&self) -> String {
        self.node.to_string()
    }

    fn pool(&self) -> String {
        self.pool.to_string()
    }

    fn name(&self) -> Option<String> {
        self.name.clone().map(|name| name.to_string())
    }

    fn uuid(&self) -> String {
        self.uuid.to_string()
    }

    fn protocol(&self) -> String {
        self.protocol.to_string()
    }
}

impl ShareReplicaInfo for ShareReplicaRequest {
    fn node(&self) -> String {
        self.node_id.clone()
    }

    fn pool(&self) -> String {
        self.pool_id.clone()
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn uuid(&self) -> String {
        self.replica_id.clone().unwrap()
    }

    fn protocol(&self) -> String {
        let protocol: ReplicaShareProtocol = self.protocol.into();
        protocol.to_string()
    }
}

pub trait UnshareReplicaInfo {
    fn node(&self) -> String;
    fn pool(&self) -> String;
    fn name(&self) -> Option<String>;
    fn uuid(&self) -> String;
}

impl UnshareReplicaInfo for UnshareReplica {
    fn node(&self) -> String {
        self.node.to_string()
    }

    fn pool(&self) -> String {
        self.pool.to_string()
    }

    fn name(&self) -> Option<String> {
        self.name.clone().map(|name| name.to_string())
    }

    fn uuid(&self) -> String {
        self.uuid.to_string()
    }
}

impl UnshareReplicaInfo for UnshareReplicaRequest {
    fn node(&self) -> String {
        self.node_id.clone()
    }

    fn pool(&self) -> String {
        self.pool_id.clone()
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn uuid(&self) -> String {
        self.replica_id.clone().unwrap()
    }
}

impl From<&(dyn CreateReplicaInfo + Send + Sync)> for CreateReplicaRequest {
    fn from(data: &(dyn CreateReplicaInfo + Send + Sync)) -> Self {
        Self {
            node_id: data.node(),
            pool_id: data.pool(),
            name: data.name(),
            replica_id: Some(data.uuid()),
            thin: data.thin(),
            size: data.size(),
            share: Protocol::try_from(data.share()).unwrap().into(),
            managed: data.managed(),
            owners: Some(replica_grpc::ReplicaOwners {
                volume: data.owners().volume().map(|id| id.to_string()),
                nexuses: data
                    .owners()
                    .nexuses()
                    .iter()
                    .map(|id| id.to_string())
                    .collect(),
            }),
        }
    }
}

impl From<&(dyn DestroyReplicaInfo + Send + Sync)> for DestroyReplicaRequest {
    fn from(data: &(dyn DestroyReplicaInfo + Send + Sync)) -> Self {
        Self {
            node_id: data.node(),
            pool_id: data.pool(),
            name: data.name(),
            replica_id: Some(data.uuid()),
            owners: Some(replica_grpc::ReplicaOwners {
                volume: data.disowners().volume().map(|id| id.to_string()),
                nexuses: data
                    .disowners()
                    .nexuses()
                    .iter()
                    .map(|id| id.to_string())
                    .collect(),
            }),
        }
    }
}

impl From<&(dyn ShareReplicaInfo + Send + Sync)> for ShareReplicaRequest {
    fn from(data: &(dyn ShareReplicaInfo + Send + Sync)) -> Self {
        Self {
            node_id: data.node(),
            pool_id: data.pool(),
            name: data.name(),
            replica_id: Some(data.uuid()),
            protocol: ReplicaShareProtocol::try_from(data.protocol().as_str())
                .unwrap()
                .into(),
        }
    }
}

impl From<&(dyn UnshareReplicaInfo + Send + Sync)> for UnshareReplicaRequest {
    fn from(data: &(dyn UnshareReplicaInfo + Send + Sync)) -> Self {
        Self {
            node_id: data.node(),
            pool_id: data.pool(),
            name: data.name(),
            replica_id: Some(data.uuid()),
        }
    }
}
