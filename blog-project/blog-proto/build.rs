fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "proto/blog.proto";

    println!("cargo:rerun-if-changed={proto_file}");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&[proto_file], &["proto"])?;

    Ok(())
}
