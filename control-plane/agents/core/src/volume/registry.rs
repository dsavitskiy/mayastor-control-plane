use crate::core::registry::Registry;
use common::errors::{SvcError, VolumeNotFound};
use common_lib::types::v0::message_bus::{
    NexusStatus, ReplicaTopology, Volume, VolumeId, VolumeState, VolumeStatus,
};

use crate::core::reconciler::PollTriggerEvent;
use common_lib::types::v0::store::replica::ReplicaSpec;
use snafu::OptionExt;
use std::collections::HashMap;

impl Registry {
    /// Get the volume state for the specified volume
    pub(crate) async fn get_volume_state(
        &self,
        volume_uuid: &VolumeId,
    ) -> Result<VolumeState, SvcError> {
        let replica_specs = self.specs().get_volume_replicas(volume_uuid);
        let volume_spec = self
            .specs()
            .get_locked_volume(volume_uuid)
            .context(VolumeNotFound {
                vol_id: volume_uuid.to_string(),
            })?;
        let volume_spec = volume_spec.lock().clone();
        let nexus_spec = self.specs().get_volume_target_nexus(&volume_spec);
        let nexus_state = match nexus_spec {
            None => None,
            Some(spec) => {
                let nexus_id = spec.lock().uuid.clone();
                self.get_nexus(&nexus_id).await.ok()
            }
        };

        // Construct the topological information for the volume replicas.
        let mut replica_topology = HashMap::new();
        for spec in &replica_specs {
            let replica_spec = spec.lock().clone();
            replica_topology.insert(
                replica_spec.uuid.clone(),
                self.replica_topology(&replica_spec).await,
            );
        }

        Ok(if let Some(nexus_state) = nexus_state {
            VolumeState {
                uuid: volume_uuid.to_owned(),
                size: nexus_state.size,
                status: match nexus_state.status {
                    NexusStatus::Online
                        if nexus_state.children.len() != volume_spec.num_replicas as usize =>
                    {
                        VolumeStatus::Degraded
                    }
                    _ => nexus_state.status.clone(),
                },
                target: Some(nexus_state),
                replica_topology,
            }
        } else {
            VolumeState {
                uuid: volume_uuid.to_owned(),
                size: volume_spec.size,
                status: if volume_spec.target.is_none() {
                    if replica_specs.len() >= volume_spec.num_replicas as usize {
                        VolumeStatus::Online
                    } else if replica_specs.is_empty() {
                        VolumeStatus::Faulted
                    } else {
                        VolumeStatus::Degraded
                    }
                } else {
                    VolumeStatus::Unknown
                },
                target: None,
                replica_topology,
            }
        })
    }

    /// Construct a replica topology from a replica spec.
    /// If the replica cannot be found, return the default replica topology.
    async fn replica_topology(&self, spec: &ReplicaSpec) -> ReplicaTopology {
        match self.get_replica(&spec.uuid).await {
            Ok(state) => ReplicaTopology::new(Some(state.node), Some(state.pool), state.status),
            Err(_) => {
                tracing::trace!(replica.uuid = %spec.uuid, "Replica not found. Constructing default replica topology");
                ReplicaTopology::default()
            }
        }
    }

    /// Get all volumes
    pub(super) async fn get_volumes(&self) -> Vec<Volume> {
        let mut volumes = vec![];
        let volume_specs = self.specs().get_volumes();
        for spec in volume_specs {
            if let Ok(state) = self.get_volume_state(&spec.uuid).await {
                volumes.push(Volume::new(spec, state));
            }
        }
        volumes
    }

    /// Return a volume object corresponding to the ID.
    pub(crate) async fn get_volume(&self, id: &VolumeId) -> Result<Volume, SvcError> {
        Ok(Volume::new(
            self.specs().get_volume(id)?,
            self.get_volume_state(id).await?,
        ))
    }

    /// Notify the reconcilers if the volume is degraded
    pub(crate) async fn notify_if_degraded(&self, volume: &Volume, event: PollTriggerEvent) {
        if volume.status() == Some(VolumeStatus::Degraded) {
            self.notify(event).await;
        }
    }
}
