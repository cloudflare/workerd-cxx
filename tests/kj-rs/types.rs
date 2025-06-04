#[cxx::bridge(namespace = "kj_rs")]
pub mod ffi {
    unsafe extern "C++" {
        include!("tests/kj-rs/cxx-types.h");

        type CppType;
        fn cpptype_get(&self) -> u64;
    }
}
