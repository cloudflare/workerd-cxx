//! The `workerd-cxx` module containing the [`Own<T>`] type, which is bindings to the `kj::Own<T>` C++ type
pub mod repr {
    use std::ffi::c_void;
    use std::fmt::{self, Debug, Display};
    use std::hash::{Hash, Hasher};
    use std::marker::PhantomData;
    use std::ops::Deref;
    use std::ops::DerefMut;
    use std::pin::Pin;

    use crate::ffi;

    /// A [`Own<T>`] represents the `kj::Own<T>`. It is a smart pointer to an opaque C++ type.
    #[repr(C)]
    pub struct Own<T> {
        disposer: *const c_void,
        ptr: *mut T,
        _ty: PhantomData<T>,
    }
    /// Public-facing Own api
    impl<T> Own<T> {
        /// Returns a mutable pinned reference to the object owned by this [`Own`]
        /// if any, otherwise None.
        pub fn as_mut(&mut self) -> Option<Pin<&mut T>> {
            unsafe {
                let mut_reference = self.ptr.as_mut()?;
                Some(Pin::new_unchecked(mut_reference))
            }
        }

        /// Returns a reference to the object owned by this [`Own`] if any,
        /// otherwise None.
        #[must_use]
        pub fn as_ref(&self) -> Option<&T> {
            unsafe { self.ptr.as_ref() }
        }

        /// Returns a mutable pinned reference to the object owned by this
        /// [`Own`].
        ///
        /// # Panics
        ///
        /// Panics if the [`Own`] holds a null pointer.
        ///
        /// ```compile_fail
        /// let mut own = ffi::cxx_kj_own();
        /// let pin1 = own.pin_mut();
        /// let pin2 = own.pin_mut();
        /// pin1.set_data(12); // Causes a compile fail, because we invalidated the first borrow
        /// ```
        ///
        /// ```compile_fail
        ///
        /// let mut own = ffi::cxx_kj_own();
        /// let pin = own.pin_mut();
        /// let moved  = own;
        /// own.set_data(143); // Compile fail, because we tried using a moved object
        /// ```
        pub fn pin_mut(&mut self) -> Pin<&mut T> {
            match self.as_mut() {
                Some(target) => target,
                None => {
                    panic!("called pin_mut on a null Own");
                }
            }
        }

        /// Returns a raw const pointer to the object owned by this [`Own`] if
        /// any, otherwise the null pointer.
        #[must_use]
        pub fn as_ptr(&self) -> *const T {
            self.ptr.cast()
        }
    }

    unsafe impl<T> Send for Own<T> where T: Send {}

    unsafe impl<T> Sync for Own<T> where T: Sync {}

    impl<T> Deref for Own<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            match self.as_ref() {
                Some(target) => target,
                None => panic!("called deref on a null Own"),
            }
        }
    }

    impl<T> DerefMut for Own<T>
    where
        T: Unpin,
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            match self.as_mut() {
                Some(target) => Pin::into_inner(target),
                None => panic!("called deref_mut on a null Own"),
            }
        }
    }

    // Own is not a self-referential type and is safe to move out of a Pin,
    // regardless whether the pointer's target is Unpin.
    impl<T> Unpin for Own<T> where T: {}

    impl<T> Debug for Own<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Own(ptr: {:p}, disposer: {:p})", self.ptr, self.disposer)
        }
    }

    impl<T> Display for Own<T>
    where
        T: Display,
    {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            match self.as_ref() {
                None => formatter.write_str("nullptr"),
                Some(value) => Display::fmt(value, formatter),
            }
        }
    }

    impl<T> PartialEq for Own<T>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.as_ref() == other.as_ref()
        }
    }

    impl<T> Eq for Own<T> where T: Eq {}

    impl<T> PartialOrd for Own<T>
    where
        T: PartialOrd,
    {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            PartialOrd::partial_cmp(&self.as_ref(), &other.as_ref())
        }
    }

    impl<T> Ord for Own<T>
    where
        T: Ord,
    {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            Ord::cmp(&self.as_ref(), &other.as_ref())
        }
    }

    impl<T> Hash for Own<T>
    where
        T: Hash,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.as_ref().hash(state);
        }
    }

    impl<T> Drop for Own<T> {
        fn drop(&mut self) {
            let this = std::ptr::from_mut::<Self>(self).cast::<ffi::OwnVoid>();
            unsafe {
                ffi::destroy_own(this);
            }
        }
    }
}
