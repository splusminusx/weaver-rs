extern crate tower_grpc_build;

fn main() {
    tower_grpc_build::Config::new()
        .enable_server(true)
        .enable_client(true)
        .build(&["protocol/weaver.proto"], &["protocol/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
