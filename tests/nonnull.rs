use std::ptr::NonNull;

#[cxx::bridge(namespace = "tests")]
pub mod ffi {
    struct Resource {
        value: i32,
    }

    unsafe extern "C++" {
        include!("tests/nonnull_tests.h");

        type NativeResource;

        // Test passing NonNull from Rust to C++
        unsafe fn take_resource_nonnull(ptr: NonNull<Resource>);

        // Test returning a raw pointer from C++ as NonNull on Rust side
        unsafe fn create_resource() -> NonNull<Resource>;

        // Test reading value through NonNull
        unsafe fn get_resource_value(ptr: NonNull<Resource>) -> i32;
    }
}

#[test]
fn test_nonnull_pass_to_cpp() {
    // Create a Resource on Rust side
    let mut resource = ffi::Resource { value: 42 };

    // Get NonNull pointer
    let nonnull_ptr = NonNull::new(&mut resource as *mut ffi::Resource).unwrap();

    // Pass to C++ - should automatically convert to *mut Resource
    unsafe {
        ffi::take_resource_nonnull(nonnull_ptr);
    }
}

#[test]
fn test_nonnull_return_from_cpp() {
    // Get NonNull from C++ function
    let nonnull_ptr = unsafe { ffi::create_resource() };

    // NonNull guarantees non-null, so we can directly use it
    // Read the value
    let value = unsafe { ffi::get_resource_value(nonnull_ptr) };
    assert_eq!(value, 100);
}

#[test]
fn test_nonnull_roundtrip() {
    // Create resource from C++
    let nonnull_ptr = unsafe { ffi::create_resource() };

    // Verify the initial value
    let value = unsafe { ffi::get_resource_value(nonnull_ptr) };
    assert_eq!(value, 100);

    // Note: We don't pass it to take_resource_nonnull because that expects value=42
    // This test just verifies that roundtrip (C++ -> Rust -> C++) works for reading
}
