use std::env;

#[test]
fn generate_proto_code() {
    env::set_var("OUT_DIR", "src");
    prost_build::compile_protos(&["src/proto/bank_msgs.proto"],
                                &["src/"]).unwrap();
}