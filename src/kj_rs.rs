//! Temporary module to hold the rust side of the kj::Own<T> type.

use std::mem::MaybeUninit;
use std::fmt::{self, Display, Debug};
use std::ffi::c_void;
use std::marker::PhantomData;

/// A [`KjOwn<T>`] represents the `kj::Own<T>`. It is a smart pointer to an opaque C++ type.
/// TODO: More Docs
#[repr(C)]
pub struct KjOwn<T>
where
    T: KjOwnTarget
{
    repr: [MaybeUninit<*mut c_void>; 2],
    _ty: PhantomData<T>    
}

// Possible Other Types:
// - SpaceFor<T>/construct>

// Public-facing KjOwn api, backed by calls to unsafe code generated for each [`KjOwnTarget`]
impl<T> KjOwn<T>
where
    T: KjOwnTarget
{
    // Possible functions can include:
    // [`KjOwn::heap`]
    // [`KjOwn::fakeOwn`]
    // [`KjOwn::refCounted`]
    // [`KjOwn::attachRef`]

    /// Returns a reference to the object owned by this KjOwn if any,
    /// otherwise None.
    pub fn as_ref(&self) -> Option<&T> {
        let this = self as *const Self as *const c_void;
        unsafe { T::__get(this).as_ref() }
    }
}

/// Represents a type which can be held in a [`KjOwn`] smart pointer
/// TODO: More Docs
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

// CXX SharedPtr boilerplate:
// 
// KjOwn is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.

impl<T> Unpin for KjOwn<T> where T: KjOwnTarget {}

// impl<T> Debug for KjOwn<T>
// where
//     T: Debug + KjOwnTarget,
// {
//     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         match self.as_ref() {
//             None => formatter.write_str("nullptr"),
//             Some(value) => Debug::fmt(value, formatter),
//         }
//     }
// }

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
        let this = self as *mut Self as *mut c_void;
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
