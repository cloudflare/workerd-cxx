#[cxx::bridge(namespace = "kj_rs")]
pub mod ffi {
    unsafe extern "C++" {
        include!("tests/kj-rs/tests.h");

        async fn c_async_fn() -> impl Future<Output = ()>;
    }
}


#[cfg(test)]
mod tests {
    use crate::ffi;

    #[test]
    fn foo() {
        let _fut = ffi::c_async_fn();
        assert_eq!(1, 1);
    }
}
