fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/fhe")
        .compile(
            &["protos/fhe.proto"], &[""]
        ).unwrap();
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/frodo_pir")
        .compile(
            &["protos/frodo_pir.proto"], &[""]
        ).unwrap();
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/double_pir")
        .compile(
            &["protos/double_pir.proto"], &[""]
        ).unwrap();
    Ok(())
}