pub mod repr {
    use std::mem::MaybeUninit;

    use static_assertions::assert_eq_size;
    pub struct Niche;

    pub trait IsNull {
        fn is_null(&self) -> bool;
    }

    impl<T> IsNull for *const T {
        fn is_null(&self) -> bool {
            <*const T>::is_null(*self)
        }
    }

    impl<T> IsNull for *mut T {
        fn is_null(&self) -> bool {
            <*mut T>::is_null(*self)
        }
    }

    impl<T> IsNull for &T {
        fn is_null(&self) -> bool {
            false
        }
    }

    impl<T> IsNull for &mut T {
        fn is_null(&self) -> bool {
            false
        }
    }

    #[repr(C)]
    pub struct Maybe<T, D = bool> {
        is_set: D,
        some: MaybeUninit<T>,
    }

    impl<T> Maybe<T, bool> {
        pub fn is_some(&self) -> bool {
            return self.is_set;
        }

        pub fn is_none(&self) -> bool {
            return !self.is_set;
        }
    }

    impl<T: IsNull> Maybe<T, Niche> {
        pub fn is_some(&self) -> bool {
            return !self.is_none();
        }

        pub fn is_none(&self) -> bool {
            unsafe {
                return self.some.assume_init_read().is_null();
            }
        }
    }

    // TODO: For when `Own<T>` gets merged
    // impl<T> Maybe<crate::repr::Own<T>, Ptr> {
    //     
    // }

    assert_eq_size!(Maybe<isize>, [usize; 2]);
    assert_eq_size!(Maybe<&isize, Niche>, usize);

    // #[repr(C, usize)]
    // pub enum Maybe<T> {
    //     None,
    //     Some(T),
    // }

    // #[repr(C)]
    // pub struct Maybe<T, D = bool> {
    //     is_set: D,
    //     value: MaybeUninit<T>,
    // }

    // impl<T> Maybe<T> {
    //     pub fn is_none(&self) -> bool {
    //         matches!(self, Maybe::None)
    //     }

    //     pub fn is_some(&self) -> bool {
    //         matches!(self, Maybe::Some(_))
    //     }
    // }

    // impl<T> Maybe<T> {
    //     pub fn is_none(&self) -> bool {
    //         !self.is_set
    //     }

    //     pub fn is_some(&self) -> bool {
    //         self.is_set
    //     }
    // }

    impl<T> From<Maybe<T>> for Option<T> {
        fn from(value: Maybe<T>) -> Self {
            if value.is_some() {
                unsafe {
                    Some(value.some.assume_init())
                }
            } else {
                None
            }
        }
    }

    // impl<T> From<Option<T>> for Maybe<T> {
    //     fn from(value: Option<T>) -> Self {
    //         match value {
    //             None => Maybe {
    //                 is_set: false,
    //                 value: MaybeUninit::uninit(),
    //             },
    //             Some(val) => Maybe {
    //                 is_set: true,
    //                 value: MaybeUninit::from(val),
    //             },
    //         }
    //     }
    // }

    // impl<T> Drop for Maybe<T> {
    //     fn drop(&mut self) {
    //         if self.is_set {
    //             unsafe {
    //                 self.value.assume_init_drop();
    //             }
    //         }
    //     }
    // }
}
