//! Module for both [`KjRc`] and [`KjArc`], since they're nearly identical types

// Allows using `Refcounted` and `AtomicRefcounted` from `kj_rs::refcounted` as
// if it was a Rust trait representing a refcounted object.
pub use repr::{AtomicRefcounted, Refcounted};

pub mod repr {
    use crate::KjOwn;
    use std::ffi::c_void;
    use std::marker::PhantomData;
    use std::ops::Deref;
    use std::pin::Pin;

    /// # Safety
    /// Should only be automatically implemented by the bridge macro
    pub unsafe trait Refcounted {
        fn is_shared(&self) -> bool;
        /// # Safety
        /// Do not call this function, instead, clone the [`KjRc`].
        unsafe fn add_ref(rc: &KjRc<Self>) -> KjRc<Self>;
        /// # Safety
        /// `rc` must point to a valid C++ `kj::Rc<T>` object.
        unsafe fn get_ptr(rc: &KjRc<Self>) -> *const Self;
        /// # Safety
        /// `rc` must point to a valid C++ `kj::Rc<T>` object.
        unsafe fn drop_rc(rc: *mut KjRc<Self>);
    }

    /// # Safety
    /// Should only be automatically implemented by the bridge macro
    pub unsafe trait AtomicRefcounted {
        fn is_shared(&self) -> bool;
        /// # Safety
        /// Do not call this function, instead, clone the [`KjArc`].
        unsafe fn add_ref(arc: &KjArc<Self>) -> KjArc<Self>;
    }

    /// Bindings to the kj type `kj::Rc`. Represents and owned and reference counted type,
    /// like Rust's [`std::rc::Rc`].
    #[repr(C)]
    pub struct KjRc<T: Refcounted + ?Sized> {
        // Must match the in-memory representation of C++ `kj::Rc<T>`.
        _raw_rc: *const c_void,
        _ty: PhantomData<*const T>,
    }

    /// Bindings to the kj type `kj::Arc`. Represents and owned and atomically reference
    /// counted type, like Rust's [`std::sync::Arc`].
    #[repr(C)]
    pub struct KjArc<T: AtomicRefcounted + ?Sized> {
        own: KjOwn<T>,
    }

    unsafe impl<T: AtomicRefcounted> Send for KjArc<T> where T: Send {}
    unsafe impl<T: AtomicRefcounted> Sync for KjArc<T> where T: Sync {}

    impl<T: Refcounted> KjRc<T> {
        #[must_use]
        pub fn get(&self) -> *const T {
            // Safety:
            //     `self` is always created from a valid C++ `kj::Rc<T>`.
            unsafe { T::get_ptr(self) }
        }

        #[must_use]
        pub fn is_shared(&self) -> bool {
            self.deref().is_shared()
        }

        // The return value here represents exclusive access to the pointee.
        // This allows for exclusive mutation of the inner value.
        pub fn get_mut(&mut self) -> Option<Pin<&mut T>> {
            if self.is_shared() {
                None
            } else {
                // Safety:
                //     We have exclusive access to this `KjRc` and `is_shared()` is false,
                //     so no other owning aliases exist.
                unsafe {
                    self.get()
                        .cast_mut()
                        .as_mut()
                        .map(|reference| Pin::new_unchecked(reference))
                }
            }
        }
    }

    impl<T: AtomicRefcounted> KjArc<T> {
        #[must_use]
        pub fn get(&self) -> *const T {
            self.own.as_ptr()
        }

        // The return value here represents exclusive access to the internal `Own`.
        // This allows for exclusive mutation of the inner value.
        pub fn get_mut(&mut self) -> Option<&mut KjOwn<T>> {
            if self.own.is_shared() {
                None
            } else {
                Some(&mut self.own)
            }
        }
    }

    impl<T: Refcounted> Deref for KjRc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            // Safety:
            //     C++ guarantees a valid pointee for non-null `kj::Rc<T>`. This API
            //     follows the existing `KjOwn` behavior and treats null from C++ as UB.
            unsafe { self.get().as_ref().unwrap() }
        }
    }

    impl<T: AtomicRefcounted> Deref for KjArc<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.own
        }
    }

    /// Using clone to create another count, like how Rust does it.
    impl<T: Refcounted> Clone for KjRc<T> {
        fn clone(&self) -> Self {
            unsafe { T::add_ref(self) }
        }
    }

    impl<T: AtomicRefcounted> Clone for KjArc<T> {
        fn clone(&self) -> Self {
            unsafe { T::add_ref(self) }
        }
    }

    impl<T: Refcounted + ?Sized> Drop for KjRc<T> {
        fn drop(&mut self) {
            // Safety:
            //     `self` stores an initialized C++ `kj::Rc<T>` object.
            unsafe { T::drop_rc(self) }
        }
    }

    // No `Drop` needs to be implemented for `KjArc`, because the internal `Own`
    // `Drop` is sufficient.
}
