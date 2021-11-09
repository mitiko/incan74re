// build.rs

fn main() {
    cc::Build::new()
        .flag("-Wno-unused-parameter")
        .file("libs/libsais.c")
        .compile("libsais");
    println!("cargo:rerun-if-changed=libs/libsais.c");
    println!("cargo:rerun-if-changed=libs/libsais.h");
}
