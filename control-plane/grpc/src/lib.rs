pub mod client;
pub mod grpc_opts;
pub mod misc;
pub mod pool;
pub mod replica;

// Common module for all the misc operations
// TODO: move this to its respective directory structure
pub(crate) mod common {
    tonic::include_proto!("v1.common");
}

// Pool GRPC module for the autogenerated pool code
pub(crate) mod pool_grpc {
    tonic::include_proto!("v1.pool");
}

// Replica GRPC module for the autogenerated replica code
pub(crate) mod replica_grpc {
    tonic::include_proto!("v1.replica");
}
