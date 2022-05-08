// To build, download the libsais library (libsais.c and libsais.h from https://github.com/IlyaGrebnov/libsais) into the libs folder

// The libsais library is released under Apache License 2.0 and is not modified for the purposes of this project
// Copyright of the libsais library (c) 2021 Ilya Grebnov <ilya.grebnov@gmail.com>

// Uses libsais v2.70

fn main() {
    cc::Build::new()
        .flag("-Wno-unused-parameter")
        .file("libs/libsais.c")
        .compile("libsais");
    println!("cargo:rerun-if-changed=libs/libsais.c");
    println!("cargo:rerun-if-changed=libs/libsais.h");
}
