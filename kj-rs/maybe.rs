use repr::Maybe;
use std::mem::MaybeUninit;

/// # Safety
/// This trait should only be implemented in `workerd-cxx` on types
/// which contain a specialization of `kj::Maybe` that needs to be represented in
/// Rust.
unsafe trait HasNiche: Sized {
    fn is_niche(value: &MaybeUninit<Self>) -> bool;
}

// In Rust, references are not allowed to be null, so a null `MaybeUninit<&T>` is a niche
unsafe impl<T> HasNiche for &T {
    fn is_niche(value: &MaybeUninit<&T>) -> bool {
        unsafe {
            // This is to avoid potentially zero-initalizing a reference, which is always undefined behavior
            std::mem::transmute_copy::<MaybeUninit<&T>, MaybeUninit<*const T>>(value)
                .assume_init()
                .is_null()
        }
    }
}

unsafe impl<T> HasNiche for &mut T {
    fn is_niche(value: &MaybeUninit<&mut T>) -> bool {
        unsafe {
            // This is to avoid potentially zero-initalizing a reference, which is always undefined behavior
            std::mem::transmute_copy::<MaybeUninit<&mut T>, MaybeUninit<*mut T>>(value)
                .assume_init()
                .is_null()
        }
    }
}

// In `kj`, `kj::Own<T>` are considered `none` in a `Maybe` if the pointer is null
unsafe impl<T> HasNiche for crate::repr::Own<T> {
    fn is_niche(value: &MaybeUninit<crate::repr::Own<T>>) -> bool {
        unsafe { value.assume_init_ref().as_ptr().is_null() }
    }
}

/// Trait that is used as the bounds for what can be in a Maybe
///
/// # Safety
/// This trait should only be implemented from macro expansion and should
/// never be manually implemented.
pub unsafe trait MaybeItem: Sized {
    type Discriminant: Copy;
    fn is_some(value: &Maybe<Self>) -> bool;
    fn is_none(value: &Maybe<Self>) -> bool;
    fn from_option(value: Option<Self>) -> Maybe<Self>;
    fn drop_in_place(value: &mut Maybe<Self>) {
        if <Self as MaybeItem>::is_some(value) {
            unsafe {
                value.some.assume_init_drop();
            }
        }
    }
}

/// Macro to implement [`MaybeItem`] for `T` which implment [`HasNiche`]
macro_rules! impl_maybe_item_for_has_niche {
    ($ty:ty) => {
        unsafe impl<T> MaybeItem for $ty {
            type Discriminant = ();

            fn is_some(value: &Maybe<Self>) -> bool {
                !<$ty as HasNiche>::is_niche(&value.some)
            }

            fn is_none(value: &Maybe<Self>) -> bool {
                <$ty as HasNiche>::is_niche(&value.some)
            }

            fn from_option(value: Option<Self>) -> Maybe<Self> {
                match value {
                    None => Maybe {
                        is_set: (),
                        some: MaybeUninit::zeroed(),
                    },
                    Some(val) => Maybe {
                        is_set: (),
                        some: MaybeUninit::new(val),
                    }
                }
            }
        }
    };
    ($ty:ty, $($tail:ty),+) => {
        impl_maybe_item_for_has_niche!($ty);
        impl_maybe_item_for_has_niche!($($tail),*);
    };
}

/// Macro to implement [`MaybeItem`] for primitives
macro_rules! impl_maybe_item_for_primitive {
    ($ty:ty) => {
        unsafe impl MaybeItem for $ty {
            type Discriminant = bool;

            fn is_some(value: &Maybe<Self>) -> bool {
                value.is_set
            }

            fn is_none(value: &Maybe<Self>) -> bool {
                !value.is_set
            }

            fn from_option(value: Option<Self>) -> Maybe<Self> {
                match value {
                    None => Maybe {
                        is_set: false,
                        some: MaybeUninit::zeroed(),
                    },
                    Some(val) => Maybe {
                        is_set: true,
                        some: MaybeUninit::new(val),
                    }
                }
            }
        }
    };
    ($ty:ty, $($tail:ty),+) => {
        impl_maybe_item_for_primitive!($ty);
        impl_maybe_item_for_primitive!($($tail),*);
    };
}

impl_maybe_item_for_has_niche!(crate::Own<T>, &T, &mut T);
impl_maybe_item_for_primitive!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, char, bool
);

pub(crate) mod repr {
    use super::MaybeItem;
    use static_assertions::assert_eq_size;
    use std::fmt::{Debug, Display};
    use std::mem::MaybeUninit;

    /// A [`Maybe`] represents bindings to the `kj::Maybe` class.
    /// It is an optional type, but represented using a struct, for alignment with kj.
    ///
    /// # Layout
    /// In kj, `Maybe` has 3 specializations, one without niche value optimization, and
    /// two with it. In order to maintain an identical layout in Rust, we include an associated type
    /// in the [`MaybeItem`] trait, which determines the discriminant of the `Maybe<T: MaybeItem>`.
    ///
    /// ## Niche Value Optimization
    /// This discriminant is used in tandem with the [`crate::maybe::HasNiche`] to implement
    /// [`MaybeItem`] properly for values which have a niche, which use a discriminant of [`()`],
    /// the unit type. All other types use [`bool`].
    #[repr(C)]
    pub struct Maybe<T: MaybeItem> {
        pub(super) is_set: T::Discriminant,
        pub(super) some: MaybeUninit<T>,
    }

    assert_eq_size!(Maybe<isize>, [usize; 2]);
    assert_eq_size!(Maybe<&isize>, usize);
    assert_eq_size!(Maybe<crate::Own<isize>>, [usize; 2]);

    impl<T: MaybeItem> Maybe<T> {
        /// # Safety
        /// This function shouldn't be used except by macro generation.
        pub unsafe fn is_set(&self) -> T::Discriminant {
            self.is_set
        }

        /// # Safety
        /// This function shouldn't be used except by macro generation.
        pub unsafe fn from_parts_unchecked(
            is_set: T::Discriminant,
            some: MaybeUninit<T>,
        ) -> Maybe<T> {
            Maybe { is_set, some }
        }

        pub fn is_some(&self) -> bool {
            T::is_some(self)
        }

        pub fn is_none(&self) -> bool {
            T::is_none(self)
        }
    }

    impl<T: MaybeItem> From<Maybe<T>> for Option<T> {
        fn from(value: Maybe<T>) -> Self {
            if value.is_some() {
                // We can't move out of value so we copy it and forget it in
                // order to perform a "manual" move out of value
                let ret = unsafe { Some(value.some.assume_init_read()) };
                std::mem::forget(value);
                ret
            } else {
                None
            }
        }
    }

    impl<T: MaybeItem> From<Option<T>> for Maybe<T> {
        fn from(value: Option<T>) -> Self {
            <T as MaybeItem>::from_option(value)
        }
    }

    impl<T: MaybeItem + Debug> Debug for Maybe<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.is_none() {
                write!(f, "kj::None")
            } else {
                write!(f, "kj::Some({:?})", unsafe { self.some.assume_init_ref() })
            }
        }
    }

    impl<T: MaybeItem + Display> Display for Maybe<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.is_none() {
                write!(f, "kj::None")
            } else {
                write!(f, "kj::Some({})", unsafe { self.some.assume_init_ref() })
            }
        }
    }

    impl<T: MaybeItem> Drop for Maybe<T> {
        fn drop(&mut self) {
            T::drop_in_place(self);
        }
    }
}
