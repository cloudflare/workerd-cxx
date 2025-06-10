#[cxx::bridge(namespace = "kj_rs")]
pub mod ffi {
    unsafe extern "C++" {
        include!("tests/kj-rs/cxx-types.h");

        type CppType;
        fn cpptype_get(&self) -> u64;
        fn cpptype_set(self: Pin<&mut CppType>, val: u64);
    }

    // Necessary for cxx to generate the `KjOwn` bindings, as the use
    // of `KjOwn` is in a different bridge module
    impl KjOwn<CppType> {}
}
