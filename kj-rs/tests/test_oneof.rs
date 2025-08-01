#[cfg(test)]
pub mod tests {
    use crate::ffi;
    use kj_rs::repr::OneOf;

    #[test]
    fn test_pass() {
        let oneof = ffi::new_oneof();
        assert_eq!(oneof.tag(), 1);
        assert_eq!(unsafe { *oneof.get() }, 12);
    }
}
