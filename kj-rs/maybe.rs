pub mod repr {
    use std::mem::MaybeUninit;

    #[repr(C)]
    pub struct Maybe<T> {
        is_set: bool,
        value: MaybeUninit<T>
    }
}
