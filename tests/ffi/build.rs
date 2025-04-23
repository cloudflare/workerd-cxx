#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use cxx_build::CFG;

fn main() {
    if cfg!(trybuild) {
        return;
    }

    CFG.include_prefix = "tests/ffi";
    let sources = vec!["lib.rs", "module.rs"];
    let mut build = cxx_build::bridges(sources);
    build.compiler("/usr/bin/clang-20");
    build.file("tests.cc");
    build.std("c++20");
    build.warnings_into_errors(cfg!(deny_warnings));
    build.include("/home/maizatskyi/src/github.com/capnproto/kj-rs/src/");
    build.include("/home/maizatskyi/src/github.com/capnproto/capnproto/c++/src/");
    if cfg!(not(target_env = "msvc")) {
        build.define("CXX_TEST_INSTANTIATIONS", None);
    }
    build.compile("cxx-test-suite");

    println!("cargo:rerun-if-changed=tests.cc");
    println!("cargo:rerun-if-changed=tests.h");
}
