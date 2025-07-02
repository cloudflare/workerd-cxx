#[cfg(test)]
pub mod tests {
    use crate::ffi;
    use kj_rs::repr::Maybe;

    #[test]
    fn test_shared() {
        let maybe: Maybe<i64> = ffi::return_maybe();
        println!("{}", unsafe { std::mem::transmute_copy::<Maybe<i64>, [u8; 16]>(&maybe) }
            .into_iter()
            .map(|c| format!("{c:02}"))
            .collect::<String>());
        assert!(!maybe.is_none());
    }
}
