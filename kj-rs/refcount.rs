//! Module for both [`KjRc`] and [`KjArc`], since they're nearly identical types

pub mod repr {
    use crate::Own;
    use std::ops::Deref;

    /// # Safety
    /// - Should only be automatically implemented by the bridge macro
    pub unsafe trait Refcounted {
        fn is_shared(&self) -> bool;
        fn add_ref(rc: &KjRc<Self>) -> KjRc<Self>;
    }
    /// # Safety
    /// - Should only be automatically implemented by the bridge macro
    pub unsafe trait AtomicRefcounted {
        fn is_shared(&self) -> bool;
        fn add_ref(arc: &KjArc<Self>) -> KjArc<Self>;
    }

    #[repr(C)]
    pub struct KjRc<T: Refcounted + ?Sized> {
        own: Own<T>,
    }

    // TODO: `Send` and `Sync`
    #[repr(C)]
    pub struct KjArc<T: AtomicRefcounted + ?Sized> {
        own: Own<T>,
    }

    impl<T: Refcounted> KjRc<T> {
        #[must_use]
        pub fn get(&self) -> *const T {
            self.own.as_ptr()
        }
    }

    impl<T: AtomicRefcounted> KjArc<T> {
        #[must_use]
        pub fn get(&self) -> *const T {
            self.own.as_ptr()
        }
    }

    impl<T: Refcounted> Deref for KjRc<T> {
        type Target = Own<T>;

        fn deref(&self) -> &Self::Target {
            &self.own
        }
    }

    impl<T: AtomicRefcounted> Deref for KjArc<T> {
        type Target = Own<T>;

        fn deref(&self) -> &Self::Target {
            &self.own
        }
    }

    impl<T: Refcounted> Clone for KjRc<T> {
        fn clone(&self) -> Self {
            T::add_ref(self)
        }
    }

    impl<T: AtomicRefcounted> Clone for KjArc<T> {
        fn clone(&self) -> Self {
            T::add_ref(self)
        }
    }
}
