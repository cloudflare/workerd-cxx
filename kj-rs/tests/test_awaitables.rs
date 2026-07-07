//! Tests for KJ awaitables driven from Rust's `#[tokio::test]`.
//!
//! These tests mirror the C++ tests in `awaitables-cc-test.c++` but run from Rust using
//! [`kj_rs::runtime::KjRuntime`] to drive the KJ event loop alongside Tokio.

use kj_rs::runtime::KjRuntime;

use crate::ffi;

// ---------------------------------------------------------------------------
// Ready futures / promises
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ready_future_void() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        crate::test_futures::new_ready_future_void().await;
    })
    .await;
}

#[tokio::test]
async fn ready_promise_void() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        ffi::new_ready_promise_void()
            .await
            .expect("ready promise should not throw");
    })
    .await;
}

#[tokio::test]
async fn ready_promise_i32() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        let value = ffi::new_ready_promise_i32(42)
            .await
            .expect("should not throw");
        assert_eq!(value, 42);
    })
    .await;
}

// ---------------------------------------------------------------------------
// Layered: Rust .await of KJ promises
// ---------------------------------------------------------------------------

#[tokio::test]
async fn layered_ready_future_void() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        crate::test_futures::new_layered_ready_future_void()
            .await
            .expect("layered ready future should succeed");
    })
    .await;
}

// ---------------------------------------------------------------------------
// Naive select: poll multiple KJ promises simultaneously
// ---------------------------------------------------------------------------

#[tokio::test]
async fn naive_select_future_void() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        crate::test_futures::new_naive_select_future_void()
            .await
            .expect("naive select should succeed");
    })
    .await;
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn errored_promise_void() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        let err = ffi::new_errored_promise_void()
            .await
            .expect_err("should throw");
        assert!(
            err.what().contains("test error"),
            "error should contain 'test error', got: {}",
            err.what()
        );
    })
    .await;
}

#[tokio::test]
async fn error_handling_future_infallible() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        crate::test_futures::new_error_handling_future_void_infallible().await;
    })
    .await;
}

// ---------------------------------------------------------------------------
// Awaiting typed promises
// ---------------------------------------------------------------------------

#[tokio::test]
async fn awaiting_future_i32() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        crate::test_futures::new_promise_i32_awaiting_future_void()
            .await
            .expect("awaiting i32 future should succeed");
    })
    .await;
}

// ---------------------------------------------------------------------------
// Waker plumbing tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn wrapped_waker_future_void() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        crate::test_futures::new_wrapped_waker_future_void()
            .await
            .expect("wrapped waker future should succeed");
    })
    .await;
}
