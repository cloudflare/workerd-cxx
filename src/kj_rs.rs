//! Temporary module to hold the rust side of the kj::Own<T> type.

use std::mem::MaybeUninit;
use std::ffi::c_void;
use std::marker::PhantomData;

/// A [`KjOwn<T>`] represents the `kj::Own<T>`. It is a smart pointer to an opaque C++ type.
/// TODO: More Docs
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
}

/// Represents a type which can be held in a [`KjOwn`] smart pointer
/// TODO: More Docs
pub unsafe trait KjOwnTarget {
    #[doc(hidden)]
    unsafe fn __drop(repr: *mut c_void);    
}

unsafe impl<T> Send for KjOwn<T> where T: Send + KjOwnTarget {}

unsafe impl<T> Sync for KjOwn<T> where T: Sync + KjOwnTarget {}

// CXX SharedPtr boilerplate:
// 
// KjOwn is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.

impl<T> Unpin for KjOwn<T> where T: KjOwnTarget {}


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
