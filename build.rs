#![allow(deprecated)]
extern crate gcc;

fn main() {
    gcc::Build::new()
        .cpp(true) // Switch to C++ library compilation.
        .file("src/SpookyV2.cpp")
        .include("src")
        .compile("libSpookyV2.a");
}