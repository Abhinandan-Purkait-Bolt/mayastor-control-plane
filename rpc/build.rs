use std::{path::Path, process::Command};

extern crate tonic_build;

fn main() {
    if !Path::new("mayastor-api/.git").exists() {
        let output = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .output()
            .expect("failed to execute git command ");

        if !output.status.success() {
            panic!("submodule checkout failed");
        }
    }

    tonic_build::configure()
        .build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(
            &["mayastor-api/mayastor/protobuf/mayastor.proto"],
            &["mayastor-api/mayastor/protobuf"],
        )
        .unwrap_or_else(|e| panic!("mayastor protobuf compilation failed: {}", e));

    tonic_build::configure()
        .build_server(true)
        .compile(
            &["mayastor-api/mayastor/protobuf/csi.proto"],
            &["mayastor-api/mayastor/protobuf"],
        )
        .unwrap_or_else(|e| panic!("CSI protobuf compilation failed: {}", e));

    tonic_build::configure()
        .compile(
            &["mayastor-api/mayastor-control-plane/protobuf/v1/node.proto"],
            &["mayastor-api/mayastor-control-plane/protobuf/v1/"],
        )
        .unwrap_or_else(|e| panic!("Node protobuf compilation failed: {}", e));
}
