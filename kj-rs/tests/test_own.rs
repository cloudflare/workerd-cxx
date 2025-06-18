use crate::ffi;

// Send and Sync are explicity opt-in when using cxx.
// # Safety
// The type `OqaqueCxxClass` contains no raw pointers or interior mutability.
// It is a wrapper around an integer, and thus safe to send between threads.
unsafe impl Send for ffi::OpaqueCxxClass {}
// # Safety
// `OpaqueCxxClass` cannot be unsafely mutated from a shared reference.
unsafe impl Sync for ffi::OpaqueCxxClass {}

#[cfg(test)]
pub mod tests {
    use crate::ffi;

    #[test]
    fn kj_own() {
        let mut own = ffi::cxx_kj_own();
        // Methods on C++ classes can be called from Rust
        assert_eq!(own.get_data(), 42);
        own.pin_mut().set_data(14);
        assert_eq!(own.get_data(), 14);
        // Explicitly drop for clarity / debugging drop impl
        std::mem::drop(own);
    }

    #[test]
    fn kj_move() {
        let mut owned = ffi::cxx_kj_own();
        owned.pin_mut().set_data(27);
        // Move owned into moved_value
        let moved_value = owned;
        let moved_value2 = moved_value;
        let mut moved_value3 = moved_value2;
        assert_eq!(moved_value3.get_data(), 27);
        moved_value3.pin_mut().set_data(14);
        assert_eq!(moved_value3.get_data(), 14);
    }

    #[test]
    fn test_pass_own_back() {
        let mut own = ffi::cxx_kj_own();
        own.pin_mut().set_data(14);
        ffi::give_own_back(own);
    }

    #[test]
    fn kj_get_test() {
        let mut own = ffi::cxx_kj_own();
        let data_ptr = own.as_ptr();
        own.pin_mut().set_data(75193);
        unsafe {
            assert_eq!(data_ptr.as_ref().unwrap().get_data(), 75193);
        }
    }

    #[test]
    fn test_breaking_things() {
        let _ = ffi::breaking_things();
    }

    #[test]
    #[should_panic]
    fn test_panic_null() {
        let own = ffi::null_kj_own();
        let _ = own.get_data();
    }

    #[test]
    fn modify_own_return_test_rust() {
        ffi::modify_own_return_test();
    }

    #[test]
    fn test_own_as_ref_as_mut() {
        let mut own = ffi::cxx_kj_own();

        // Test as_ref
        assert!(own.as_ref().is_some());
        assert_eq!(own.as_ref().unwrap().get_data(), 42);

        // Test as_mut
        assert!(own.as_mut().is_some());
        own.as_mut().unwrap().set_data(99);
        assert_eq!(own.get_data(), 99);

        // Test null own
        let null_own = ffi::null_kj_own();
        assert!(null_own.as_ref().is_none());
    }

    #[test]
    fn test_own_as_ptr() {
        let own = ffi::cxx_kj_own();
        let ptr = own.as_ptr();
        assert!(!ptr.is_null());

        let null_own = ffi::null_kj_own();
        let null_ptr = null_own.as_ptr();
        assert!(null_ptr.is_null());
    }

    #[test]
    fn test_own_in_collections() {
        let mut owns_vec = Vec::new();
        let mut owns_map = std::collections::HashMap::new();

        // Test in Vec
        for i in 0..10 {
            let mut own = ffi::cxx_kj_own();
            own.pin_mut().set_data(i * 10);
            owns_vec.push(own);
        }

        // Test in HashMap
        for i in 0..10 {
            let mut own = ffi::cxx_kj_own();
            own.pin_mut().set_data(i * 100);
            owns_map.insert(format!("key_{i}"), own);
        }

        // Verify all values
        for (i, own) in owns_vec.iter().enumerate() {
            assert_eq!(own.get_data(), (i * 10) as u64);
        }

        for i in 0u64..10 {
            let key = format!("key_{i}");
            assert_eq!(owns_map[&key].get_data(), i * 100);
        }
    }

    #[test]
    #[should_panic(expected = "called deref on a null Own")]
    fn test_null_own_deref_panic() {
        let null_own = ffi::null_kj_own();
        let _ = null_own.get_data(); // Should panic via Deref
    }

    #[test]
    #[should_panic(expected = "called pin_mut on a null Own")]
    fn test_null_own_pin_mut_panic() {
        let mut null_own = ffi::null_kj_own();
        null_own.pin_mut().set_data(42); // Should panic
    }

    #[test]
    fn test_own_rapid_create_destroy() {
        for _ in 0..1000 {
            let mut own = ffi::cxx_kj_own();
            own.pin_mut().set_data(12345);
            assert_eq!(own.get_data(), 12345);
            // Implicit drop at end of loop
        }
    }

    #[test]
    fn test_own_nested_operations() {
        let mut own1 = ffi::cxx_kj_own();
        let mut own2 = ffi::cxx_kj_own();

        own1.pin_mut().set_data(100);
        own2.pin_mut().set_data(200);

        // Use one own's value to modify another
        let val1 = own1.get_data();
        own1.pin_mut().set_data(val1 + own2.get_data());

        assert_eq!(own1.get_data(), 300);
        assert_eq!(own2.get_data(), 200);
    }

    #[test]
    fn test_own_debug_display() {
        let own = ffi::cxx_kj_own();
        let debug_str = format!("{own:?}");
        assert!(!debug_str.is_empty());

        let null_own = ffi::null_kj_own();
        let null_debug = format!("{null_own:?}");
        assert_eq!(null_debug, "Own { ptr: 0x0, disposer: 0x0 }");
    }

    #[test]
    fn test_own_send_between_threads() {
        use std::thread;

        let own = ffi::cxx_kj_own();
        let handle = thread::spawn(move || {
            // Own should be Send, so this should work
            assert_eq!(own.get_data(), 42);
            own
        });

        let returned_own = handle.join().unwrap();
        assert_eq!(returned_own.get_data(), 42);
    }

    #[test]
    fn test_own_concurrent_creation() {
        use std::sync::Arc;
        use std::sync::Barrier;
        use std::thread;

        let num_threads = 10;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = Vec::new();

        for thread_id in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Create and use Own concurrently
                let mut own = ffi::cxx_kj_own();
                own.pin_mut().set_data(thread_id as u64 * 100);
                assert_eq!(own.get_data(), thread_id as u64 * 100);

                // Return the final value for verification
                own.get_data()
            });
            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // Verify all threads completed successfully
        results.sort_unstable();
        let expected: Vec<u64> = (0..num_threads).map(|i| i as u64 * 100).collect();
        assert_eq!(results, expected);
    }

    // This is one test generated by Claude. I am unsure it sufficiently tests multithreading.
    #[test]
    fn test_own_stress_multithreaded() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        let num_threads: u64 = 12;
        let items_per_thread: u64 = 100;

        for thread_id in 0..num_threads {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                for i in 0..items_per_thread {
                    let mut own = ffi::cxx_kj_own();
                    let value = thread_id * items_per_thread + i;
                    own.pin_mut().set_data(value);

                    // Send the Own across thread boundary
                    tx_clone.send(own).unwrap();
                }
            });
        }
        drop(tx); // Close the sending side

        // Collect all Owns from all threads
        let mut received_owns = Vec::new();
        while let Ok(own) = rx.recv() {
            received_owns.push(own);
        }

        // Verify we received the expected number
        assert_eq!(
            received_owns.len(),
            (num_threads * items_per_thread) as usize
        );

        // Verify all values are correct
        let mut values: Vec<u64> = received_owns.iter().map(|own| own.get_data()).collect();
        values.sort_unstable();

        let expected: Vec<u64> = (0..(num_threads * items_per_thread)).collect();
        assert_eq!(values, expected);
    }
}
