use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include = PathBuf::from("include");
    let protos = PathBuf::from("protos");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed={}", include.display());
    println!("cargo:rerun-if-changed={}", protos.display());

    // gen bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include.display()))
        .clang_arg("-xc++") // Parse as C++ to handle C++11 typed enums
        .clang_arg("-std=c++17") // Use C++17 standard
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_default(true)
        .generate()
        .expect("unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("unable to write bindings!");

    // gen protobuf
    prost_build::Config::new()
        .include_file("protos.rs")
        .compile_protos(
            &[protos.join("xla/pjrt/proto/compile_options.proto")],
            &[protos],
        )
        .expect("unable to compile protos");
}
