extern crate tonic_build;

fn main() {
    tonic_build::configure()
        .compile(
            &["proto/pool/v1/pool.proto", "proto/misc/common.proto"],
            &["proto/"],
        )
        .unwrap();
}
