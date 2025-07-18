use crate::ffi::{OpaqueCxxClass, Shared};
use kj_rs::repr::{Maybe, Own};

pub fn take_maybe_own_ret(val: Maybe<Own<OpaqueCxxClass>>) -> Maybe<Own<OpaqueCxxClass>> {
    let mut option: Option<Own<OpaqueCxxClass>> = val.into();
    if let Some(val) = &mut option {
        val.as_mut().set_data(42);
    }

    option.into()
}

pub fn take_maybe_own(val: Maybe<Own<OpaqueCxxClass>>) {
    let option: Option<Own<OpaqueCxxClass>> = val.into();
    // Own gets destoyed at end of `if let` block, because it takes ownership of `option`
    if let Some(own) = option {
        assert_eq!(own.get_data(), 42);
    }
}

/// # Safety: Uses a reference in a function that can be called from C++, which is opaque
/// to the Rust compiler, so it cannot verify lifetime requirements
pub unsafe fn take_maybe_ref_ret<'a>(val: Maybe<&'a u64>) -> Maybe<&'a u64> {
    let option: Option<&u64> = val.into();
    if let Some(num) = &option {
        assert_eq!(**num, 15);
    }
    option.into()
}

pub fn take_maybe_ref(val: Maybe<&u64>) {
    let mut option: Option<&u64> = val.into();
    // Pure Rust at this point, but just in case
    if let Some(val) = option.take() {
        assert_eq!(*val, 15);
    }
}

pub fn take_maybe_shared_ret(val: Maybe<Shared>) -> Maybe<Shared> {
    let mut option: Option<Shared> = val.into();
    if let Some(mut shared) = option.take() {
        shared.i = 18;
    }
    option.into()
}

pub fn take_maybe_shared(val: Maybe<Shared>) {
    let _: Option<Shared> = val.into();
}

#[cfg(test)]
pub mod tests {
    use crate::ffi::{self, OpaqueCxxClass, Shared};
    use kj_rs::repr::{Maybe, Own};

    #[test]
    fn test_some() {
        let maybe: Maybe<i64> = ffi::return_maybe();
        assert!(!maybe.is_none());
    }

    #[test]
    fn test_none() {
        let maybe: Maybe<i64> = ffi::return_maybe_none();
        assert!(maybe.is_none());
    }

    #[test]
    fn test_none_ref() {
        let maybe = ffi::return_maybe_ref_none();
        assert!(maybe.is_none());
    }

    #[test]
    fn test_none_ref_opt() {
        let maybe = ffi::return_maybe_ref_none();
        let maybe: Option<&i64> = maybe.into();
        assert!(maybe.is_none());
    }

    #[test]
    fn test_some_ref() {
        let maybe = ffi::return_maybe_ref_some();
        assert!(maybe.is_some());
    }

    #[test]
    fn test_some_ref_opt() {
        let maybe = ffi::return_maybe_ref_some();
        let maybe: Option<&i64> = maybe.into();
        assert!(maybe.is_some());
    }

    #[test]
    fn test_some_shared() {
        let maybe: Maybe<Shared> = ffi::return_maybe_shared_some();
        assert!(!maybe.is_none());
        let opt: Option<Shared> = maybe.into();
        assert!(opt.is_some());
        assert_eq!(opt.unwrap().i, 14);
    }

    #[test]
    fn test_none_shared() {
        let maybe: Maybe<Shared> = ffi::return_maybe_shared_none();
        assert!(maybe.is_none());
        let opt: Option<Shared> = maybe.into();
        assert!(opt.is_none());
    }

    #[test]
    fn test_some_own() {
        let maybe = ffi::return_maybe_own_some();
        assert!(!maybe.is_none());
        let opt: Option<Own<OpaqueCxxClass>> = maybe.into();
        assert!(opt.is_some());
        assert_eq!(opt.unwrap().get_data(), 14);
    }

    #[test]
    fn test_none_own() {
        let maybe = ffi::return_maybe_own_none();
        assert!(maybe.is_none());
        let opt: Option<Own<OpaqueCxxClass>> = maybe.into();
        assert!(opt.is_none());
    }

    #[test]
    fn test_some_own_maybe() {
        let maybe = ffi::return_maybe_own_some();
        assert!(!maybe.is_none());
        assert!(maybe.is_some());
    }

    #[test]
    fn test_none_own_maybe() {
        let maybe = ffi::return_maybe_own_none();
        assert!(maybe.is_none());
        assert!(!maybe.is_some());
    }

    #[test]
    fn test_maybe_driver() {
        ffi::test_maybe_reference_shared_own_driver();
    }
}
