use std::convert::TryFrom;
//use std::env;

pub mod grpc_opts;
pub mod pool;
pub mod replica;
pub mod server;

//use dns_lookup::lookup_host;
use tonic::transport::Uri;

//const CORE_HOSTNAME: &str = "core";

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

pub fn get_core_ip() -> Uri {
    // let addr = match lookup_host(CORE_HOSTNAME) {
    //     Ok(ipv4addr) => ipv4addr[0].to_string(),
    //     Err(_) => env::var("CORE_IP").expect("CORE_IP not found"),
    // };
    // Uri::try_from(format!("https://{}:50051", addr)).unwrap()
    Uri::try_from("https://10.1.0.5:50051").unwrap()
}
