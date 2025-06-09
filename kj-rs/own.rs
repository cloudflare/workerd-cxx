//! The `workerd-cxx` module containing the [`KjOwn<T>`] type, which is bindings to the `kj::Own<T>` C++ type
use crate::fmt::display;
use std::ffi::c_void;
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;

/// A [`KjOwn<T>`] represents the `kj::Own<T>`. It is a smart pointer to an opaque C++ type.
#[repr(C)]
pub struct KjOwn<T>
where
    T: KjOwnTarget,
{
    repr: [MaybeUninit<*mut c_void>; 2],
    _ty: PhantomData<T>,
}

// Possible Other Types:
// - SpaceFor<T>/construct>

// Public-facing KjOwn api, backed by calls to unsafe code generated for each [`KjOwnTarget`]
impl<T> KjOwn<T>
where
    T: KjOwnTarget,
{
    // Possible functions can include:
    // [`KjOwn::heap`] for allocating C++ types from rust, by calling into the C++ `kj::heap` function
    // [`KjOwn::fakeOwn`]
    // [`KjOwn::refCounted`]
    // [`KjOwn::attachRef`]

    /// Returns a mutable pinned reference to the object owned by this [`KjOwn`]
    /// if any, otherwise None.
    pub fn as_mut(&mut self) -> Option<Pin<&mut T>> {
        let this = std::ptr::from_ref::<Self>(self).cast::<c_void>();
        unsafe {
            let mut_reference = T::__get(this).cast_mut().as_mut()?;
            Some(Pin::new_unchecked(mut_reference))
        }
    }

    /// Returns a reference to the object owned by this [`KjOwn`] if any,
    /// otherwise None.
    #[must_use]
    pub fn as_ref(&self) -> Option<&T> {
        let this = std::ptr::from_ref::<Self>(self).cast::<c_void>();
        unsafe { T::__get(this).as_ref() }
    }

    /// Returns a mutable pinned reference to the object owned by this
    /// [`KjOwn`].
    ///
    /// # Panics
    ///
    /// Panics if the [`KjOwn`] holds a null pointer.
    pub fn pin_mut(&mut self) -> Pin<&mut T> {
        match self.as_mut() {
            Some(target) => target,
            None => panic!("called pin_mut on a null KjOwn<{}>", display(T::__typename),),
        }
    }

    /// Returns a raw const pointer to the object owned by this [`KjOwn`] if
    /// any, otherwise the null pointer.
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        let this = std::ptr::from_ref::<Self>(self).cast::<c_void>();
        unsafe { T::__get(this) }
    }
}

/// Represents a type which can be held in a [`KjOwn`] smart pointer.
/// # Safety
/// Cannot be implmented outside of generated workerd-cxx code.
pub unsafe trait KjOwnTarget {
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    unsafe fn __drop(repr: *mut c_void);
    #[doc(hidden)]
    unsafe fn __get(this: *const c_void) -> *const Self;
}

unsafe impl<T> Send for KjOwn<T> where T: Send + KjOwnTarget {}

unsafe impl<T> Sync for KjOwn<T> where T: Sync + KjOwnTarget {}

impl<T> Deref for KjOwn<T>
where
    T: KjOwnTarget,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!("called deref on a null KjOwn<{}>", display(T::__typename),),
        }
    }
}

impl<T> DerefMut for KjOwn<T>
where
    T: KjOwnTarget + Unpin,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.as_mut() {
            Some(target) => Pin::into_inner(target),
            None => panic!(
                "called deref_mut on a null KjOwn<{}>",
                display(T::__typename),
            ),
        }
    }
}

// KjOwn is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.
impl<T> Unpin for KjOwn<T> where T: KjOwnTarget {}

impl<T> Debug for KjOwn<T>
where
    T: Debug + KjOwnTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
    }
}

impl<T> Display for KjOwn<T>
where
    T: Display + KjOwnTarget,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Display::fmt(value, formatter),
        }
    }
}

impl<T> Drop for KjOwn<T>
where
    T: KjOwnTarget,
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
        unsafe impl KjOwnTarget for $ty {
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
