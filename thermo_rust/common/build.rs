fn main() {
   // Make generated files available for IDE, by generating them in the source dir:
   let out_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/pb");
   std::fs::create_dir_all(&out_path).expect("Failed to create directory for generated files");

   tonic_build::configure()
      .out_dir(out_path)
      .protoc_arg("--experimental_allow_proto3_optional")
      .compile_protos(&["agg.proto"], &["proto"])
      .expect("Failed to compile protos");
}
