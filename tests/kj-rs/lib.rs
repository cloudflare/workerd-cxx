#[cxx::bridge(namespace = "kj_rs")]
pub mod ffi {
    struct Shared {
        i: i64,
    }

    unsafe extern "C++" {
        include!("tests/kj-rs/tests.h");

        type CppType;

        async fn c_async_void_fn();

        // todo
        // async fn c_async_int_fn() -> i64;
        // async fn c_async_struct_fn() -> Shared;

        fn cpp_kj_own() -> KjOwn<CppType>;
        fn cpptype_get(&self) -> u64;
        fn call();
    }

}


#[cfg(test)]
mod tests {
    use crate::ffi;

    // let kj-rs verify the behavior, just check compilation
    #[allow(clippy::let_underscore_future)]
    #[test]
    fn compilation() {
        let _ =  ffi::c_async_void_fn();
        // let _ =  ffi::c_async_int_fn();
        // let _ =  ffi::c_async_struct_fn();
    }

    #[test]
    fn kj_own() {
        ffi::call();
        let own = ffi::cpp_kj_own();
        assert_eq!(own.cpptype_get(), 42);
        std::mem::drop(own);
    }
}
