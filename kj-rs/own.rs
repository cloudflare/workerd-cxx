//! The `workerd-cxx` module containing the [`Own<T>`] type, which is bindings to the `kj::Own<T>` C++ type
use std::ffi::c_void;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;

/// A [`Own<T>`] represents the `kj::Own<T>`. It is a smart pointer to an opaque C++ type.
#[repr(C)]
pub struct Own<T>
where
    T: OwnTarget,
{
    disposer: *const c_void,
    ptr: *mut T,
    // repr: [MaybeUninit<*mut c_void>; 2],
    _ty: PhantomData<T>,
}

// Possible Other Types:
// - SpaceFor<T>/construct>

/// Public-facing Own api, backed by calls to unsafe code generated for each [`OwnTarget`]
impl<T> Own<T>
where
    T: OwnTarget,
{
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
        unsafe {
            self.ptr.as_ref()
        }
    }

    /// Returns a mutable pinned reference to the object owned by this
    /// [`Own`].
    ///
    /// # Panics
    ///
    /// Panics if the [`Own`] holds a null pointer.
    pub fn pin_mut(&mut self) -> Pin<&mut T> {
        match self.as_mut() {
            Some(target) => target,
            None => {
                panic!("called pin_mut on a null Own<{}>", T::__typename());
            },
        }
    }

    /// Returns a raw const pointer to the object owned by this [`Own`] if
    /// any, otherwise the null pointer.
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.cast()
    }
}

/// Represents a type which can be held in a [`Own`] smart pointer.
/// # Safety
/// Cannot be implmented outside of generated workerd-cxx code.
pub unsafe trait OwnTarget {
    #[doc(hidden)]
    fn __typename() -> &'static str;
    #[doc(hidden)]
    unsafe fn __drop(repr: *mut c_void);
}

unsafe impl<T> Send for Own<T> where T: Send + OwnTarget {}

unsafe impl<T> Sync for Own<T> where T: Sync + OwnTarget {}

impl<T> Deref for Own<T>
where
    T: OwnTarget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!("called deref on a null Own<{}>", T::__typename()),
        }
    }
}

impl<T> DerefMut for Own<T>
where
    T: OwnTarget + Unpin,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.as_mut() {
            Some(target) => Pin::into_inner(target),
            None => panic!("called deref_mut on a null Own<{}>", T::__typename()),
        }
    }
}

// Own is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.
impl<T> Unpin for Own<T> where T: OwnTarget {}

impl<T> Debug for Own<T>
where
    T: OwnTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Own")
            .field("ptr", &self.ptr)
            .field("disposer", &self.disposer)
            .finish()
    }
}

impl<T> Display for Own<T>
where
    T: Display + OwnTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Display::fmt(value, formatter),
        }
    }
}

impl<T> Drop for Own<T>
where
    T: OwnTarget,
{
    fn drop(&mut self) {
        let this = std::ptr::from_mut::<Self>(self).cast::<c_void>();
        unsafe { T::__drop(this) }
    }
}

// TODO: Generate bindings for primitive ffi-safe types
// Must include the drop shim manually for each included type.
// (Drop for primitives should be a no-op)
macro_rules! impl_own_target {
    ($ty:ty) => {
        impl_own_target!($ty, stringify!($ty), stringify!($ty))
    };
    ($ty:ty, $name:expr, $segment:expr) => {
        unsafe impl OwnTarget for $ty {
            fn __typename() -> &'static str {
                $name
            }
            unsafe fn __drop(this: *mut c_void) {
                extern "C" {
                    // NOTE: the "cxxbridge1$std" prefix means the binding is *not* automatic
                    #[link_name = concat!("cxxbridge1$std$kjown$", $segment, "$drop")]
                    fn __drop(this: *mut c_void);
                }
                unsafe { __drop(this) }
            }
        }
    };
}
