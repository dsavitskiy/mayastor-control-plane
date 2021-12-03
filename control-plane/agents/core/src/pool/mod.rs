mod registry;
pub mod service;
pub mod specs;

use std::{convert::TryInto, marker::PhantomData, sync::Arc};

use super::{core::registry::Registry, handler, impl_request_handler};
use async_trait::async_trait;
use common::{errors::SvcError, handler::*, Service};

// Replica Operations
use common_lib::types::v0::message_bus::{
    CreateReplica, DestroyReplica, GetReplicas, ShareReplica, UnshareReplica,
};

pub(crate) async fn configure(builder: Service) -> Service {
    let registry = builder.get_shared_state::<Registry>().clone();
    let new_service = service::Service::new(registry);
    builder
        .with_channel(ChannelVs::Pool)
        .with_default_liveness()
        .with_shared_state(new_service.clone())
        .with_pool_transport(Arc::new(new_service))
        .await
        .with_subscription(handler!(GetReplicas))
        .with_subscription(handler!(CreateReplica))
        .with_subscription(handler!(DestroyReplica))
        .with_subscription(handler!(ShareReplica))
        .with_subscription(handler!(UnshareReplica))
}

/// Pool Agent's Tests
#[cfg(test)]
mod tests;
