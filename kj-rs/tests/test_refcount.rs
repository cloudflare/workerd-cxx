#[cfg(test)]
pub mod tests {
    use crate::ffi;

    #[test]
    fn test_rc() {
        let rc = ffi::get_rc();
        assert_eq!(rc.get_data(), 15);
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.get_data(), 15);
    }

    #[test]
    fn test_arc() {
        let arc = ffi::get_arc();
        assert_eq!(arc.get_data(), 16);
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.get_data(), 16);
    }
}
