use std::mem::MaybeUninit;

/// A ZST that allows for niche value optimization using a hybrid of typestate and generic programming
pub struct Niche;

/// Restrict [`Maybet`] to only `bool`s and [`Niche`]'s.
/// # Safety
/// - Do not implement this trait under any conditions.
pub unsafe trait Discriminant {}

unsafe impl Discriminant for bool {}
unsafe impl Discriminant for Niche {}

/// Trait to specify how to detect niches for niche value pointers. Should not be exposed publicly.
/// # Safety
/// - The niche for types which implement this trait must be equivalent to `MaybeUninit::zeroed()`
/// - Do not implement this trait. It should only ever be used by `workerd-cxx`
pub unsafe trait HasNiche: Sized {
    fn is_niche(value: &MaybeUninit<Self>) -> bool;
}

// NOTE: kj does NPO for raw pointers in Maybe so we do here too
unsafe impl<T> HasNiche for *const T {
    fn is_niche(value: &MaybeUninit<*const T>) -> bool {
        unsafe { value.assume_init().is_null() }
    }
}

unsafe impl<T> HasNiche for *mut T {
    fn is_niche(value: &MaybeUninit<*mut T>) -> bool {
        unsafe { value.assume_init().is_null() }
    }
}

unsafe impl<T> HasNiche for &T {
    fn is_niche(value: &MaybeUninit<&T>) -> bool {
        unsafe {
            std::mem::transmute_copy::<MaybeUninit<&T>, MaybeUninit<*const T>>(value)
                // This is to avoid potentially zero-initalizing a reference, which is always undefined behavior
                .assume_init()
                .is_null()
        }
    }
}

unsafe impl<T> HasNiche for &mut T {
    fn is_niche(value: &MaybeUninit<&mut T>) -> bool {
        unsafe {
            std::mem::transmute_copy::<MaybeUninit<&mut T>, MaybeUninit<*mut T>>(value)
                // This is to avoid potentially zero-initalizing a reference, which is always undefined behavior
                .assume_init()
                .is_null()
        }
    }
}

pub(crate) mod repr {
    use super::{Discriminant, HasNiche, Niche};
    use static_assertions::assert_eq_size;
    use std::fmt::{Debug, Display};
    use std::mem::MaybeUninit;

    /// A [`Maybe`] represents bindings to the `kj::Maybe` class.
    /// It is an optional type, but represented using a struct, for alignment with kj.
    ///
    /// # Layout
    /// In kj, `Maybe` has 3 specializations, one without niche value optimization, and
    /// two with it. In order to maintain an identical layout in Rust, we include a second
    /// generic argument with a default value of `bool`, which functions as a flag for whether
    /// the `Maybe` is some or none.
    ///
    /// ## Niche Value Optimization
    /// In kj, however, references which may be null are stored using a `kj::Maybe`, and utilize
    /// the niche value optimization. To accomplish this in Rust, we create a struct, [`Niche`],
    /// which has zero size, and is used as both the generic value of the discriminant, and as
    /// a typestate generic to specify that our [`Maybe`] utilizes niche value optimization.
    #[repr(C)]
    pub struct Maybe<T: Sized, D: Discriminant = bool> {
        is_set: D,
        some: MaybeUninit<T>,
    }

    assert_eq_size!(Maybe<isize>, [usize; 2]);
    assert_eq_size!(Maybe<&isize, Niche>, usize);

    // pub unsafe trait MaybeItem: Sized {
    //     unsafe fn __drop<D: Discriminant>(item: *mut Maybe<Self, D>);
    // }

    impl<T> Maybe<T, bool> {
        pub fn is_some(&self) -> bool {
            self.is_set
        }

        pub fn is_none(&self) -> bool {
            !self.is_set
        }
    }

    impl<T: HasNiche> Maybe<T, Niche> {
        pub fn is_some(&self) -> bool {
            !T::is_niche(&self.some)
        }

        pub fn is_none(&self) -> bool {
            T::is_niche(&self.some)
        }
    }

    // Explicitly disallow [`HasNiche`] with `bool`
    // FIXME: Replace with trait that contains all [`Maybe`] functions?
    // impl<T: HasNiche> Maybe<T, bool> {
    //     // Include debug implementation that panics and says ERROR?
    //     pub fn is_some(&self) -> bool {
    //         panic!()
    //     }

    //     pub fn is_none(&self) -> bool {
    //         panic!()
    //     }
    // }

    unsafe impl<T> HasNiche for crate::repr::Own<T> {
        fn is_niche(value: &MaybeUninit<crate::repr::Own<T>>) -> bool {
            unsafe { value.assume_init_ref().as_ptr().is_null() }
        }
    }

    impl<T> From<Maybe<T>> for Option<T> {
        fn from(value: Maybe<T>) -> Self {
            if value.is_some() {
                unsafe { Some(value.some.assume_init_read()) }
            } else {
                None
            }
        }
    }

    impl<T: HasNiche> From<Maybe<T, Niche>> for Option<T> {
        fn from(value: Maybe<T, Niche>) -> Self {
            if value.is_some() {
                unsafe { Some(value.some.assume_init_read()) }
            } else {
                None
            }
        }
    }

    impl<T> From<Option<T>> for Maybe<T> {
        fn from(value: Option<T>) -> Self {
            match value {
                None => Maybe {
                    is_set: false,
                    some: MaybeUninit::zeroed(),
                },
                Some(val) => Maybe {
                    is_set: true,
                    some: MaybeUninit::new(val),
                },
            }
        }
    }

    impl<T: HasNiche> From<Option<T>> for Maybe<T, Niche> {
        fn from(value: Option<T>) -> Self {
            match value {
                None => Maybe {
                    is_set: Niche,
                    some: MaybeUninit::zeroed(),
                },
                Some(val) => Maybe {
                    is_set: Niche,
                    some: MaybeUninit::new(val),
                },
            }
        }
    }

    impl<T: HasNiche + Debug> Debug for Maybe<T, Niche> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.is_none() {
                write!(f, "kj::None")
            } else {
                write!(f, "kj::Some({:?})", unsafe { self.some.assume_init_ref() })
            }
        }
    }

    impl<T: HasNiche + Display> Display for Maybe<T, Niche> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.is_none() {
                write!(f, "kj::None")
            } else {
                write!(f, "kj::Some({})", unsafe { self.some.assume_init_ref() })
            }
        }
    }

    // impl<T: MaybeItem, D: Discriminant> Drop for Maybe<T, D> {
    //     fn drop(&mut self) {
    //         T::__drop(self as *mut Self);
    //     }
    // }
}
