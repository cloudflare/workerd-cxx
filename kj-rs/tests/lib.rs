#![allow(clippy::needless_lifetimes)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unused_async)]

mod test_futures;
mod test_own;

use test_futures::{
    new_awaiting_future_i32, new_error_handling_future_void_infallible, new_errored_future_void,
    new_layered_ready_future_void, new_naive_select_future_void, new_pending_future_void,
    new_ready_future_i32, new_ready_future_void, new_threaded_delay_future_void,
    new_waking_future_void, new_wrapped_waker_future_void,
};
use test_own::modify_own_return;

type Result<T> = std::io::Result<T>;
type Error = std::io::Error;

#[cxx::bridge(namespace = "kj_rs_demo")]
mod ffi {
    // -----------------------------------------------------
    // Test functions

    // Helper functions to create Promises for testing purposes.
    unsafe extern "C++" {
        include!("kj-rs-demo/test-promises.h");

        async fn new_ready_promise_void();
        async fn new_pending_promise_void();
        async fn new_coroutine_promise_void();

        async fn new_errored_promise_void();
        async fn new_ready_promise_i32(value: i32) -> i32;
    }

    // Helper functions to test `kj_rs::Own`
    unsafe extern "C++" {
        include!("kj-rs-demo/cxx-types.h");
        type OpaqueCxxClass;

        #[cxx_name = "getData"]
        fn get_data(&self) -> u64;
        #[cxx_name = "setData"]
        fn set_data(self: Pin<&mut OpaqueCxxClass>, val: u64);

        fn cxx_kj_own() -> Own<OpaqueCxxClass>;
        fn null_kj_own() -> Own<OpaqueCxxClass>;
        fn give_own_back(own: Own<OpaqueCxxClass>);
        fn modify_own_return_test();
        fn breaking_things() -> Own<OpaqueCxxClass>;
    }

    // Helper function to test moving `Own` to C++
    extern "Rust" {
        fn modify_own_return(cpp_own: Own<OpaqueCxxClass>) -> Own<OpaqueCxxClass>;
    }

    enum CloningAction {
        None,
        CloneSameThread,
        CloneBackgroundThread,
        WakeByRefThenCloneSameThread,
    }

    enum WakingAction {
        None,
        WakeByRefSameThread,
        WakeByRefBackgroundThread,
        WakeSameThread,
        WakeBackgroundThread,
    }

    // Helper functions to create BoxFutureVoids for testing purposes.
    extern "Rust" {
        async fn new_pending_future_void();
        async fn new_ready_future_void();
        async fn new_waking_future_void(cloning_action: CloningAction, waking_action: WakingAction);
        async fn new_threaded_delay_future_void();
        async fn new_layered_ready_future_void() -> Result<()>;

        async fn new_naive_select_future_void() -> Result<()>;
        async fn new_wrapped_waker_future_void() -> Result<()>;

        async fn new_errored_future_void() -> Result<()>;
        async fn new_error_handling_future_void_infallible();

        async fn new_awaiting_future_i32() -> Result<()>;
        async fn new_ready_future_i32(value: i32) -> Result<i32>;
    }

    // these are used to check compilation only
    extern "Rust" {

        async unsafe fn lifetime_arg_void<'a>(buf: &'a [u8]);
        async unsafe fn lifetime_arg_result<'a>(buf: &'a [u8]) -> Result<()>;
    }
}

pub async fn lifetime_arg_void<'a>(_buf: &'a [u8]) {}

pub async fn lifetime_arg_result<'a>(_buf: &'a [u8]) -> Result<()> {
    Ok(())
}
