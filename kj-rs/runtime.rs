//! A KJ event loop that can be driven as a Rust [`Future`].
//!
//! This module wraps `kj::EventLoop` so that Rust async code can `.await` KJ promises. The
//! [`KjRuntime`] type owns a KJ event loop and provides [`KjRuntime::run()`], which accepts an
//! arbitrary `Future` and drives the KJ event loop alongside it.
//!
//! # Example
//!
//! ```ignore
//! #[tokio::test]
//! async fn test_kj_promise() {
//!     let mut kj = KjRuntime::new();
//!     kj.run(async {
//!         ffi::some_kj_promise().await.unwrap();
//!     }).await;
//! }
//! ```

use crate::ffi;
use cxx::UniquePtr;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// TODO(cleanup): Replace UniquePtr<ffi::KjRuntimeImpl> with KjOwn<ffi::KjRuntimeImpl> once the CXX
// proc macro can resolve KjOwn from within the kj_rs crate (it currently expands to
// `::kj_rs::repr::KjOwn` which doesn't work inside the kj_rs crate itself). This would let us
// remove the manual `unsafe impl Send` on KjRuntime, since KjOwn<T> is Send when T: Send.

// Safety: KJ EventLoops are designed to be transferrable between threads. The thread-local binding
// is managed entirely by kj::WaitScope, which is created and destroyed via enterScope()/leaveScope()
// within each poll_with() call. Between polls, the EventLoop is unbound and safe to move. Code
// running on the EventLoop honors the KJ convention of using EventLoop-locals instead of
// thread-locals.
unsafe impl Send for ffi::KjRuntimeImpl {}

/// A KJ event loop that can be driven as a Rust [`Future`].
///
/// Create one with [`KjRuntime::new()`], then call [`KjRuntime::run()`] to drive a Rust future
/// with the KJ event loop active on the current thread.
///
/// The event loop is bound to the calling thread only during [`KjRuntime::run()`]'s internal
/// `poll()` calls (via enterScope/leaveScope which manage a `kj::WaitScope`). Between polls, the
/// runtime is unbound and can be migrated to a different thread by the async runtime (e.g.,
/// Tokio's work-stealing scheduler).
pub struct KjRuntime {
    inner: UniquePtr<ffi::KjRuntimeImpl>,
}

// Safety: KjRuntime is Send because KjRuntimeImpl is Send (see above) and UniquePtr is just a
// pointer wrapper. The EventLoop is unbound between poll() calls, so it is safe to move between
// threads.
unsafe impl Send for KjRuntime {}

impl KjRuntime {
    /// Create a new KJ event loop.
    ///
    /// The event loop is not bound to any thread until [`run()`](KjRuntime::run) is called.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ffi::newKjRuntime(),
        }
    }

    /// Run a future with this KJ event loop active on the current thread.
    ///
    /// The future can `.await` KJ promises. The event loop is driven between each poll of the
    /// future, processing KJ events (coroutine steps, promise resolutions, cross-thread wakes).
    ///
    /// Returns when the future completes.
    pub async fn run<F: Future>(&mut self, f: F) -> F::Output {
        let mut f = std::pin::pin!(f);
        std::future::poll_fn(move |cx| self.poll_with(f.as_mut(), cx)).await
    }

    fn poll_with<F: Future>(
        &mut self,
        mut f: Pin<&mut F>,
        cx: &mut Context<'_>,
    ) -> Poll<F::Output> {
        // Bind the EventLoop to the current thread for the duration of this poll.
        // This creates a kj::WaitScope, setting the thread-local EventLoop pointer so that
        // KJ promise awaiters (RustPromiseAwaiter) can register Events on the loop.
        self.inner.pin_mut().enterScope();

        // Use a scope guard to ensure leaveScope() is always called, even on panic.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.poll_inner(f.as_mut(), cx)
        }));

        // Unbind the EventLoop from the current thread.
        self.inner.pin_mut().leaveScope();

        match result {
            Ok(poll) => poll,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    fn poll_inner<F: Future>(
        &mut self,
        mut f: Pin<&mut F>,
        cx: &mut Context<'_>,
    ) -> Poll<F::Output> {
        loop {
            // 1. Poll the user's future. It may .await KJ promises, which register Events on the
            //    KJ EventLoop via RustPromiseAwaiter. The EventLoop is bound to this thread
            //    (WaitScope is active), so KJ operations work correctly.
            if let Poll::Ready(val) = f.as_mut().poll(cx) {
                return Poll::Ready(val);
            }

            // 2. Drive the KJ event loop (non-blocking). This calls ws.poll() on the active
            //    WaitScope to process all queued Events. Promise resolutions during this call may
            //    invoke our Waker synchronously.
            let turns = self.inner.pin_mut().poll();

            // 3. If we processed events, or if more work was enqueued during processing, loop back
            //    and re-poll the user's future — a KJ promise it was .await-ing may now be resolved.
            if turns > 0 || self.inner.pin_mut().isRunnable() {
                continue;
            }

            // 4. Both the user's future and the KJ event loop are idle. Return Pending.
            //
            //    The user's future stored cx.waker() in a RustPromiseAwaiter (or another Waker-based
            //    mechanism). When a KJ promise resolves or a background thread calls waker.wake(),
            //    the outer async runtime will re-poll us from step 1.
            return Poll::Pending;
        }
    }
}

impl Default for KjRuntime {
    fn default() -> Self {
        Self::new()
    }
}
