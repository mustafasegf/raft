extern crate prost_build;

fn main() {
    println!("cargo:rerun-if-changed=src/message.proto");
    prost_build::compile_protos(&["src/message.proto"], &["src/"]).unwrap();
}
