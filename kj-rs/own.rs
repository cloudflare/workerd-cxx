//! The `workerd-cxx` module containing the [`Own<T>`] type, which is bindings to the `kj::Own<T>` C++ type
use crate::fmt::display;
use std::ffi::c_void;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;

/// A [`Own<T>`] represents the `kj::Own<T>`. It is a smart pointer to an opaque C++ type.
#[repr(C)]
pub struct Own<T>
where
    T: OwnTarget,
{
    repr: [MaybeUninit<*mut c_void>; 2],
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
        let this = std::ptr::from_ref::<Self>(self).cast::<c_void>();
        unsafe {
            let mut_reference = T::__get(this).cast_mut().as_mut()?;
            Some(Pin::new_unchecked(mut_reference))
        }
    }

    /// Returns a reference to the object owned by this [`Own`] if any,
    /// otherwise None.
    #[must_use]
    pub fn as_ref(&self) -> Option<&T> {
        let this = std::ptr::from_ref::<Self>(self).cast::<c_void>();
        unsafe { T::__get(this).as_ref() }
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
            None => panic!("called pin_mut on a null Own<{}>", display(T::__typename),),
        }
    }

    /// Returns a raw const pointer to the object owned by this [`Own`] if
    /// any, otherwise the null pointer.
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        let this = std::ptr::from_ref::<Self>(self).cast::<c_void>();
        unsafe { T::__get(this) }
    }
}

/// Represents a type which can be held in a [`Own`] smart pointer.
/// # Safety
/// Cannot be implmented outside of generated workerd-cxx code.
pub unsafe trait OwnTarget {
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    unsafe fn __drop(repr: *mut c_void);
    #[doc(hidden)]
    unsafe fn __get(this: *const c_void) -> *const Self;
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
            None => panic!("called deref on a null Own<{}>", display(T::__typename),),
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
            None => panic!(
                "called deref_mut on a null Own<{}>",
                display(T::__typename),
            ),
        }
    }
}

// Own is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.
impl<T> Unpin for Own<T> where T: OwnTarget {}

impl<T> Debug for Own<T>
where
    T: Debug + OwnTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
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

// This likely fails, but only matters for this file. Automatic impls
// are handled using a proc_macro in the macro crate.
macro_rules! impl_kjown_target {
    ($segment:expr, $name:expr, $ty:ty) => {
        unsafe impl OwnTarget for $ty {
            fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str($name)
            }
            unsafe fn __drop(this: *mut c_void) {
                extern "C" {
                    // NOTE: the "cxxbridge1$std" prefix means the binding is *not* automatic
                    #[link_name = concat!("cxxbridge1$std$kjown$", $segment, "$drop")]
                    fn __drop(this: *mut c_void);
                }
                unsafe { __drop(this) }
            }
            unsafe fn __get(this: *const c_void) -> *const Self {
                extern "C" {
                    #[link_name = concat!("cxxbridge1$std$kjown$", $segment, "$get")]
                    fn __get(this: *const c_void) -> *const c_void;
                }
                unsafe { __get(this) }.cast()
            }
        }
    };
}
