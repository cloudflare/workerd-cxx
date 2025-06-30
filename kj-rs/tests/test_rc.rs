use crate::ffi;

#[cfg(test)]
pub mod tests {
    use kj_rs::repr::Refcounted;

    use crate::ffi;

    #[test]
    fn test_rc_return() {
        let rc = ffi::cxx_kj_rc();
        assert!(!rc.as_ref().is_shared());
    }

    #[test]
    fn test_rc_return_shared() {
        let rc = ffi::cxx_kj_rc();
        let rc2 = rc.add_ref();
        assert!(rc.as_ref().is_shared());
    }
}
