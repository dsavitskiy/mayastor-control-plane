mod registry;
pub mod service;
pub mod specs;

use http::Uri;
use std::sync::Arc;

use super::core::registry::Registry;

use common::{handler::*, Service};

pub(crate) async fn configure(builder: Service) -> Service {
    let registry = builder.get_shared_state::<Registry>().clone();
    let grpc_addr = builder.get_shared_state::<Uri>().clone();
    let new_service = service::Service::new(registry.clone());
    builder
        .with_channel(ChannelVs::Pool)
        .with_default_liveness()
        .with_shared_state(new_service.clone())
        .with_transport(
            Arc::new(new_service.clone()),
            Arc::new(new_service.clone()),
            grpc_addr,
        )
        .await
}

/// Pool Agent's Tests
#[cfg(test)]
mod tests;
