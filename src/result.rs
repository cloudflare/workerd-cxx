use crate::exception::IntoKjException;

pub use repr::Result;

pub(crate) mod repr {
    use core::ptr::NonNull;

    use crate::{exception::repr::KjException, CanceledException, IntoKjException, KjError};

    #[repr(C)]
    /// Optional C++ exception. Represents results of calls to C++/rust functions across ffi
    /// boundaries. Uses pointer tagging: nullptr=None, 0x1=Canceled, other=KjException*
    pub struct Result {
        pub(crate) exception: *mut KjException,
    }
    const_assert_eq!(core::mem::size_of::<Result>(), 8);
    const_assert_eq!(core::mem::align_of::<Result>(), 8);

    impl Result {
        pub(crate) fn ok() -> Result {
            Self {
                exception: core::ptr::null_mut(),
            }
        }

        pub(crate) unsafe fn exception(exception: *mut KjException) -> Result {
            Self { exception }
        }

        pub(crate) fn error(error: KjError, file: &str, line: u32) -> Result {
            error.into_kj_exception(file, line).into()
        }

        #[allow(clippy::manual_dangling_ptr)]
        pub(crate) fn canceled() -> Result {
            Self {
                exception: 0x1 as *mut KjException,
            }
        }

        /// Turn a C++ exception into a Rust panic.
        ///
        /// Called by generated code after invoking an `extern "C++"` function which is
        /// not declared to return a `Result`. Such a signature has no way of reporting
        /// an exception to its caller, so the exception is converted into a panic. This
        /// is what keeps a throwing C++ function from terminating the process: the panic
        /// is caught again at the next `extern "Rust"` boundary and rethrown there as a
        /// `kj::Exception`.
        ///
        /// # Panics
        ///
        /// Panics if the C++ function threw an exception, or with a `CanceledException`
        /// payload if it threw `kj::CanceledException`.
        #[track_caller]
        pub fn panic_on_exception(self) {
            if let Err(exception) = self.into_result() {
                // Exceptions which did not originate from a KJ macro have no throw site.
                let thrown_at = if exception.line() > 0 {
                    alloc::format!(
                        " at {}:{}",
                        exception.file().to_str().unwrap_or("(unknown file)"),
                        exception.line(),
                    )
                } else {
                    alloc::string::String::new()
                };
                panic!(
                    "C++ exception in infallible ffi function{}: {:?}: {}",
                    thrown_at,
                    exception.r#type(),
                    exception.what(),
                );
            }
        }

        /// Convert into a `Result`.
        ///
        /// # Panics
        ///
        /// Panics if the result is a `CanceledException`.
        pub fn into_result(self) -> core::result::Result<(), crate::KjException> {
            let ptr = self.exception as usize;
            if ptr == 0 {
                // None
                Ok(())
            } else if ptr == 1 {
                // Canceled
                CanceledException::panic()
            } else {
                // KjException
                Err(crate::KjException {
                    err: unsafe { NonNull::new_unchecked(self.exception.cast()) },
                })
            }
        }
    }

    impl From<CanceledException> for Result {
        fn from(_val: CanceledException) -> Self {
            Result::canceled()
        }
    }

    impl From<crate::KjException> for Result {
        fn from(val: crate::KjException) -> Self {
            let val = core::mem::ManuallyDrop::new(val);
            unsafe { Result::exception(val.err.as_ptr()) }
        }
    }
}

/// Convert a Rust result into a `repr::Result` writing the value into ret if it is Ok.
pub unsafe fn r#try<T, E>(
    ret: *mut T,
    result: core::result::Result<T, E>,
    file: &str,
    line: u32,
) -> repr::Result
where
    E: IntoKjException,
{
    match result {
        Ok(ok) => {
            unsafe { core::ptr::write(ret, ok) }
            repr::Result::ok()
        }
        Err(err) => err.into_kj_exception(file, line).into(),
    }
}
