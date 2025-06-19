use std::pin::Pin;

use std::task::Context;

use crate::ffi::{GuardedRustPromiseAwaiter, GuardedRustPromiseAwaiterRepr};
use crate::waker::try_into_kj_waker_ptr;

use crate::lazy_pin_init::LazyPinInit;

// =======================================================================================
// GuardedRustPromiseAwaiter

// Safety: KJ Promises are not associated with threads, but with event loops at construction time.
// Therefore, they can be polled from any thread, as long as that thread has the correct event loop
// active at the time of the call to `poll()`. If the correct event loop is not active,
// GuardedRustPromiseAwaiter's API will panic. (The Guarded- prefix refers to the C++ class template
// ExecutorGuarded, which enforces the correct event loop requirement.)
unsafe impl Send for GuardedRustPromiseAwaiter {}

// =======================================================================================
// Await syntax for OwnPromiseNode

use crate::OwnPromiseNode;

pub struct PromiseAwaiter<Data: std::marker::Unpin> {
    node: Option<OwnPromiseNode>,
    pub(crate) data: Data,
    awaiter: LazyPinInit<GuardedRustPromiseAwaiterRepr>,
    // Safety: `option_waker` must be declared after `awaiter`, because `awaiter` contains a reference
    // to `option_waker`. This ensures `option_waker` will be dropped after `awaiter`.
    option_waker: OptionWaker,
}

impl<Data: std::marker::Unpin> PromiseAwaiter<Data> {
    pub fn new(node: OwnPromiseNode, data: Data) -> Self {
        PromiseAwaiter {
            node: Some(node),
            data,
            awaiter: LazyPinInit::uninit(),
            option_waker: OptionWaker::empty(),
        }
    }

    /// # Panics
    ///
    /// Panics if `node` is None.
    #[must_use]
    pub fn get_awaiter(mut self: Pin<&mut Self>) -> Pin<&mut GuardedRustPromiseAwaiter> {
        // On our first invocation, `node` will be Some, and `get_awaiter` will forward its
        // contents into GuardedRustPromiseAwaiter's constructor. On all subsequent invocations, `node`
        // will be None and the constructor will not run.
        let node = self.as_mut().node.take();

        // Safety: `awaiter` stores `rust_waker_ptr` and uses it to call `wake()`. Note that
        // `awaiter` is `self.awaiter`, which lives before `self.option_waker`. Since struct members
        // are dropped in declaration order, the `rust_waker_ptr` that `awaiter` stores will always
        // be valid during its lifetime.
        //
        // We pass a mutable pointer to C++. This is safe, because our use of the OptionWaker inside
        // of `std::task::Waker` is synchronized by ensuring we only allow calls to `poll()` on the
        // thread with the Promise's event loop active.
        let rust_waker_ptr = &raw mut self.as_mut().option_waker;

        // Safety:
        // 1. We do not implement Unpin for PromiseAwaiter.
        // 2. Our Drop trait implementation does not move the awaiter value, nor do we use
        //    `repr(packed)` anywhere.
        // 3. The backing memory is inside our pinned Future, so we can be assured our Drop trait
        //    implementation will run before Rust re-uses the memory.
        //
        // https://doc.rust-lang.org/std/pin/index.html#choosing-pinning-to-be-structural-for-field
        let awaiter = unsafe { self.map_unchecked_mut(|s| &mut s.awaiter) };

        // Safety:
        // 1. We trust that LazyPinInit's implementation passed us a valid pointer to an
        //    uninitialized GuardedRustPromiseAwaiter.
        // 2. We do not read or write to the GuardedRustPromiseAwaiter's memory, so there are no atomicity
        //    nor interleaved pointer reference access concerns.
        //
        // https://doc.rust-lang.org/std/ptr/index.html#safety
        let result = awaiter.get_or_init(move |ptr: *mut GuardedRustPromiseAwaiterRepr| unsafe {
            crate::ffi::guarded_rust_promise_awaiter_new_in_place(
                ptr as *mut GuardedRustPromiseAwaiter,
                rust_waker_ptr,
                node.expect("node should be Some in call to init()"),
            );
        });

        let awaiter = unsafe {
            let raw = Pin::into_inner_unchecked(result) as *mut GuardedRustPromiseAwaiterRepr;
            let raw = raw as *mut GuardedRustPromiseAwaiter;
            Pin::new_unchecked(&mut *raw)
        };
        awaiter
    }

    pub fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> bool {
        let maybe_kj_waker = try_into_kj_waker_ptr(cx.waker());
        let awaiter = self.as_mut().get_awaiter();
        // TODO(now): Safety comment.
        unsafe { awaiter.poll(&WakerRef(cx.waker()), maybe_kj_waker) }
    }
}

impl<Data: std::marker::Unpin> Drop for PromiseAwaiter<Data> {
    fn drop(&mut self) {
        unsafe {
            if let Some(awaiter) = self.awaiter.get() {
                crate::ffi::guarded_rust_promise_awaiter_drop_in_place(
                    awaiter as *mut GuardedRustPromiseAwaiter,
                );
            }
        }
    }
}

// =======================================================================================
// OptionWaker and WakerRef

pub struct WakerRef<'a>(&'a std::task::Waker);

// This is a wrapper around `std::task::Waker`, exposed to C++. We use it in `RustPromiseAwaiter`
// to allow KJ promises to be awaited using opaque Wakers implemented in Rust.
pub struct OptionWaker {
    inner: Option<std::task::Waker>,
}

impl OptionWaker {
    pub fn empty() -> OptionWaker {
        OptionWaker { inner: None }
    }

    pub fn set(&mut self, waker: &WakerRef) {
        if let Some(w) = &mut self.inner {
            w.clone_from(waker.0);
        } else {
            self.inner = Some(waker.0.clone());
        }
    }

    pub fn set_none(&mut self) {
        self.inner = None;
    }

    pub fn wake_mut(&mut self) {
        self.inner
            .take()
            .expect(
                "OptionWaker::set() should be called before RustPromiseAwaiter::poll(); \
                OptionWaker::wake() should be called at most once after poll()",
            )
            .wake();
    }
}
