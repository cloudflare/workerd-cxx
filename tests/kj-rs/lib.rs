#![allow(dead_code)]
#![allow(clippy::unused_async, clippy::must_use_candidate)]

pub mod types;

type Result<T> = std::io::Result<T>;

#[cxx::bridge(namespace = "kj_rs")]
pub mod ffi {
    struct Shared {
        i: i64,
    }

    unsafe extern "C++" {
        include!("tests/kj-rs/tests.h");
        type CppType = crate::types::ffi::CppType;

        async fn c_async_void_fn();
        async fn c_async_int_fn() -> i64;
        async fn c_async_struct_fn() -> Shared;

        fn cpp_kj_own() -> KjOwn<CppType>;
        fn give_own_back(own: KjOwn<CppType>);
    }

    extern "Rust" {
        async fn rust_async_void_fn();
        async fn rust_async_int_fn() -> i64;

        async fn rust_async_void_result_fn() -> Result<()>;
        async fn rust_async_int_result_fn() -> Result<i64>;
    }
}

async fn rust_async_void_fn() {}

async fn rust_async_int_fn() -> i64 {
    42
}

async fn rust_async_void_result_fn() -> Result<()> {
    Ok(())
}

async fn rust_async_int_result_fn() -> Result<i64> {
    Ok(42)
}

#[cfg(test)]
mod tests {
    use crate::ffi;

    // let kj-rs verify the behavior, just check compilation
    #[allow(clippy::let_underscore_future)]
    #[test]
    fn compilation() {
        let _ = ffi::c_async_void_fn();
        let _ = ffi::c_async_int_fn();
        let _ = ffi::c_async_struct_fn();
    }

    #[test]
    fn kj_own() {
        let mut own = ffi::cpp_kj_own();
        // Methods on C++ classes can be called from Rust
        assert_eq!(own.cpptype_get(), 42);
        own.pin_mut().cpptype_set(14);
        assert_eq!(own.cpptype_get(), 14);
        // Explicitly drop for clarity / debugging drop impl
        std::mem::drop(own);
    }

    #[test]
    fn kj_move() {
        let owned = ffi::cpp_kj_own();
        // Move owned into moved_value
        let mut moved_value = owned;
        assert_eq!(moved_value.cpptype_get(), 42);
        moved_value.pin_mut().cpptype_set(14);
        assert_eq!(moved_value.cpptype_get(), 14);
    }

    #[test]
    fn test_pass_cc() {
        let mut own = ffi::cpp_kj_own();
        own.pin_mut().cpptype_set(14);
        ffi::give_own_back(own);
    }
}
