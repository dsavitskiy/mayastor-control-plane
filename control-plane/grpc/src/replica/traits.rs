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
        CreateReplica, DestroyReplica, Filter, NexusId, Replica, ReplicaId, ReplicaOwners,
        ReplicaStatus, ShareReplica, UnshareReplica, VolumeId,
    },
};
use std::convert::TryFrom;

#[tonic::async_trait]
pub trait ReplicaOperations {
    async fn create(&self, req: CreateReplica) -> Result<Replica, ReplyError>;
    async fn get(&self, filter: Filter) -> Result<Replicas, ReplyError>;
    async fn destroy(&self, req: DestroyReplica) -> Result<(), ReplyError>;
    async fn share(&self, req: ShareReplica) -> Result<String, ReplyError>;
    async fn unshare(&self, req: UnshareReplica) -> Result<(), ReplyError>;
}

impl From<CreateReplica> for CreateReplicaRequest {
    fn from(req: CreateReplica) -> Self {
        let name = req.name.map(|name| name.to_string());
        let volume = req.owners.volume().map(|id| id.to_string());
        let replica_owners = replica_grpc::ReplicaOwners {
            volume,
            nexuses: req.owners.nexuses().iter().map(|i| i.to_string()).collect(),
        };
        CreateReplicaRequest {
            node_id: req.node.into(),
            name,
            replica_id: Some(req.uuid.into()),
            pool_id: req.pool.into(),
            thin: req.thin,
            size: req.size,
            share: req.share.into(),
            managed: req.managed,
            owners: Some(replica_owners),
        }
    }
}

impl From<CreateReplicaRequest> for CreateReplica {
    fn from(req: CreateReplicaRequest) -> Self {
        let name = req.name.map(|name| name.into());
        let volume = req
            .owners
            .clone()
            .unwrap()
            .volume
            .map(|id| VolumeId::try_from(id).unwrap());
        let replica_owners = ReplicaOwners::new(
            volume,
            req.owners
                .unwrap()
                .nexuses
                .into_iter()
                .map(|i| NexusId::try_from(i).unwrap())
                .collect(),
        );
        CreateReplica {
            node: req.node_id.into(),
            name,
            uuid: ReplicaId::try_from(req.replica_id.unwrap()).unwrap(),
            pool: req.pool_id.into(),
            size: req.size,
            thin: req.thin,
            share: req.share.into(),
            managed: req.managed,
            owners: replica_owners,
        }
    }
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

impl From<DestroyReplica> for DestroyReplicaRequest {
    fn from(req: DestroyReplica) -> Self {
        let name = req.name.map(|name| name.to_string());
        let volume = req.disowners.volume().map(|id| id.to_string());
        let replica_disowners = replica_grpc::ReplicaOwners {
            volume,
            nexuses: req
                .disowners
                .nexuses()
                .iter()
                .map(|i| i.to_string())
                .collect(),
        };
        DestroyReplicaRequest {
            node_id: req.node.into(),
            name,
            replica_id: Some(req.uuid.into()),
            pool_id: req.pool.into(),
            owners: Some(replica_disowners),
        }
    }
}

impl From<DestroyReplicaRequest> for DestroyReplica {
    fn from(req: DestroyReplicaRequest) -> Self {
        let name = req.name.map(|name| name.into());
        let volume = req
            .owners
            .clone()
            .unwrap()
            .volume
            .map(|id| VolumeId::try_from(id).unwrap());
        let replica_disowners = ReplicaOwners::new(
            volume,
            req.owners
                .unwrap()
                .nexuses
                .into_iter()
                .map(|i| NexusId::try_from(i).unwrap())
                .collect(),
        );
        DestroyReplica {
            node: req.node_id.into(),
            name,
            uuid: ReplicaId::try_from(req.replica_id.unwrap()).unwrap(),
            pool: req.pool_id.into(),
            disowners: replica_disowners,
        }
    }
}

impl From<ShareReplica> for ShareReplicaRequest {
    fn from(req: ShareReplica) -> Self {
        let name = req.name.map(|name| name.to_string());
        ShareReplicaRequest {
            node_id: req.node.into(),
            name,
            replica_id: Some(req.uuid.into()),
            pool_id: req.pool.into(),
            protocol: req.protocol.into(),
        }
    }
}

impl From<ShareReplicaRequest> for ShareReplica {
    fn from(req: ShareReplicaRequest) -> Self {
        let name = req.name.map(|name| name.into());
        ShareReplica {
            node: req.node_id.into(),
            name,
            uuid: ReplicaId::try_from(req.replica_id.unwrap()).unwrap(),
            pool: req.pool_id.into(),
            protocol: req.protocol.into(),
        }
    }
}

impl From<UnshareReplica> for UnshareReplicaRequest {
    fn from(req: UnshareReplica) -> Self {
        let name = req.name.map(|name| name.to_string());
        UnshareReplicaRequest {
            node_id: req.node.into(),
            name,
            replica_id: Some(req.uuid.into()),
            pool_id: req.pool.into(),
        }
    }
}

impl From<UnshareReplicaRequest> for UnshareReplica {
    fn from(req: UnshareReplicaRequest) -> Self {
        let name = req.name.map(|name| name.into());
        UnshareReplica {
            node: req.node_id.into(),
            name,
            uuid: ReplicaId::try_from(req.replica_id.unwrap()).unwrap(),
            pool: req.pool_id.into(),
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
