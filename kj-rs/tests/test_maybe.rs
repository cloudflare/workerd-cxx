#[cfg(test)]
pub mod tests {
    use crate::ffi;
    use kj_rs::repr::Maybe;

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
        let maybe = ffi::return_maybe_ref();
        println!("{}",
            unsafe { std::mem::transmute_copy::<Maybe<*const i64, kj_rs::repr::Niche>, [u8; 8]>(&maybe) }.into_iter()
            .map(|c| format!("{c:02x}"))
            .collect::<String>()
        );
        assert!(maybe.is_none());
        assert!(!maybe.is_some());
    }
}
