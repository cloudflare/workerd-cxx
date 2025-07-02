pub mod repr {
    use std::mem::MaybeUninit;

    use static_assertions::assert_eq_size;

    // assert_eq_size!(Maybe<isize>, [usize; 2]);
    // assert_eq_size!(Maybe<&isize>, usize);

    // #[repr(C, usize)]
    // pub enum Maybe<T> {
    //     None,
    //     Some(T),
    // }

    #[repr(C)]
    pub struct Maybe<T> {
        is_set: bool,
        value: MaybeUninit<T>
    }

    // impl<T> Maybe<T> {
        // pub fn is_none(&self) -> bool {
            // matches!(self, Maybe::None)
        // }

        // pub fn is_some(&self) -> bool {
            // matches!(self, Maybe::Some(_))
        // }
    // }

    impl<T> Maybe<T> {
        pub fn is_none(&self) -> bool {
            !self.is_set
        }

        pub fn is_some(&self) -> bool {
            self.is_set
        }
    }

    impl<T> Drop for Maybe<T> {
        fn drop(&mut self) {
            if self.is_set {
                unsafe {
                    self.value.assume_init_drop();
                }
            }
        }
    }
}
