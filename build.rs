fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/blobstore.cc")
        .file("src/ADS1X15_TLA2024.cc")
        .flag_if_supported("-std=c++14")
        .compile("ruff-hnt-rs");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/ADS1X15_TLA2024.cc");
    println!("cargo:rerun-if-changed=include/ADS1X15_TLA2024.h");
    println!("cargo:rerun-if-changed=src/blobstore.cc");
    println!("cargo:rerun-if-changed=include/blobstore.h");
}
