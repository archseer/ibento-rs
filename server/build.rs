extern crate prost_build;
extern crate tower_grpc_build;

fn main() {
    let mut prost_build = prost_build::Config::new();
    // prost_build.type_attribute("SubscribeRequest.after", "#[prost(optional)]");
    // prost_build.type_attribute("SubscribeRequest.limit", "#[prost(optional)]");

    tower_grpc_build::Config::from_prost(prost_build)
        .enable_server(true)
        .enable_client(true)
        .build(&["../proto/ibento.proto"], &["../proto"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
