#[cfg(test)]
pub mod tests {
    use crate::ffi;
    use kj_rs::repr::Maybe;

    #[test]
    fn test_shared() {
        let maybe: Maybe<i64> = ffi::shared_access(ffi::Shared { i: 14 });
        std::mem::forget(maybe);
    }
}
