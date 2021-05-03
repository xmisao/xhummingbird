extern crate protobuf_codegen_pure;

fn main() {
    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src/protos",
        input: &["protos/event.proto"],
        includes: &["protos"],
        customize: protobuf_codegen_pure::Customize {
            serde_derive: Some(true),
            ..Default::default()
        },
    })
    .expect("protoc");
}
