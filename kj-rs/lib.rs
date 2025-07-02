use awaiter::OptionWaker;
use awaiter::PtrRustPromiseAwaiter;
use awaiter::WakerRef;

pub use crate::ffi::KjWaker;
pub use awaiter::PromiseAwaiter;
pub use awaiter::RustPromiseAwaiter;
pub use future::FuturePollStatus;
pub use promise::KjPromise;
pub use promise::KjPromiseNodeImpl;
pub use promise::OwnPromiseNode;
pub use promise::PromiseFuture;
pub use promise::new_callbacks_promise_future;

mod awaiter;
mod future;
mod lazy_pin_init;
mod promise;
mod waker;

pub mod repr {
    pub use crate::future::repr::*;
}

pub type Result<T> = std::io::Result<T>;
pub type Error = std::io::Error;

#[cxx::bridge(namespace = "kj_rs")]
#[allow(clippy::needless_lifetimes)]
mod ffi {
    extern "Rust" {
        type WakerRef<'a>;
    }

    extern "Rust" {
        // We expose the Rust Waker type to C++ through this OptionWaker reference wrapper. cxx-rs
        // does not allow us to export types defined outside this crate, such as Waker, directly.
        //
        // `LazyRustPromiseAwaiter` (the implementation of `.await` syntax/the IntoFuture trait),
        // stores a OptionWaker immediately after `RustPromiseAwaiter` in declaration order.
        // pass the Waker to the `RustPromiseAwaiter` class, which is implemented in C++
        type OptionWaker;
        fn set(&mut self, waker: &WakerRef);
        fn set_none(&mut self);
        fn wake_mut(&mut self);
    }

    #[allow(clippy::should_implement_trait)]
    unsafe extern "C++" {
        include!("kj-rs/waker.h");

        // Match the definition of the abstract virtual class in the C++ header.
        type KjWaker;
        fn clone(&self) -> *const KjWaker;
        fn wake(&self);
        fn wake_by_ref(&self);
        fn drop(&self);
    }

    unsafe extern "C++" {
        include!("kj-rs/promise.h");

        type OwnPromiseNode = crate::OwnPromiseNode;

        unsafe fn own_promise_node_drop_in_place(node: *mut OwnPromiseNode);
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C++" {
        include!("kj-rs/awaiter.h");

        type RustPromiseAwaiter = crate::RustPromiseAwaiter;
        type PtrRustPromiseAwaiter = crate::PtrRustPromiseAwaiter;

        unsafe fn rust_promise_awaiter_new_in_place(
            ptr: PtrRustPromiseAwaiter,
            rust_waker_ptr: *mut OptionWaker,
            node: OwnPromiseNode,
        );
        unsafe fn rust_promise_awaiter_drop_in_place(ptr: PtrRustPromiseAwaiter);

        unsafe fn poll(self: Pin<&mut RustPromiseAwaiter>, waker: &WakerRef) -> bool;

        #[must_use]
        fn take_own_promise_node(self: Pin<&mut RustPromiseAwaiter>) -> OwnPromiseNode;
    }
}
