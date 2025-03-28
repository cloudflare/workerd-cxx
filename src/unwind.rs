#![allow(missing_docs)]
#![allow(dead_code)]

use core::mem;
use std::panic;

pub fn prevent_unwind<F, R>(label: &'static str, foreign_call: F) -> R
where
    F: FnOnce() -> R,
{
    // Goal is to make it impossible to propagate a panic across the C interface
    // of an extern "Rust" function, which would be Undefined Behavior. We
    // transform such panicks into a deterministic abort instead. When cxx is
    // built in an application using panic=abort, this guard object is compiled
    // out because its destructor is statically unreachable. When built with
    // panic=unwind, an unwind from the foreign call will attempt to drop the
    // guard object leading to a double panic, which is defined by Rust to
    // abort. In no_std programs, on most platforms the current mechanism for
    // this is for core::intrinsics::abort to invoke an invalid instruction. On
    // Unix, the process will probably terminate with a signal like SIGABRT,
    // SIGILL, SIGTRAP, SIGSEGV or SIGBUS. The precise behaviour is not
    // guaranteed and not stable, but is safe.
    let guard = Guard { label };

    let ret = foreign_call();

    // If we made it here, no uncaught panic occurred during the foreign call.
    mem::forget(guard);
    ret
}

struct Guard {
    label: &'static str,
}

impl Drop for Guard {
    #[cold]
    fn drop(&mut self) {
        panic!("panic in ffi function {}, aborting.", self.label);
    }
}

/// Run the void `foreign_call`, intercepting panics and converting them to errors.
#[cfg(feature = "std")]
pub fn catch_unwind<F>(label: &'static str, foreign_call: F) -> ::cxx::result::Result
where
    F: FnOnce(),
{
    match panic::catch_unwind(panic::AssertUnwindSafe(foreign_call)) {
        Ok(()) => ::cxx::private::Result::ok(),
        Err(err) => panic_result(label, &err),
    }
}


/// Run the error-returning `foreign_call`, intercepting panics and converting them to errors.
#[cfg(feature = "std")]
pub fn try_unwind<F>(label: &'static str, foreign_call: F) -> ::cxx::private::Result
where
    F: FnOnce() -> ::cxx::private::Result,
{
    match panic::catch_unwind(panic::AssertUnwindSafe(foreign_call)) {
        Ok(r) => r,
        Err(err) => panic_result(label, &err),
    }
}

#[cfg(feature = "std")]
fn panic_result(label: &'static str, err: &alloc::boxed::Box<dyn core::any::Any + Send>) -> ::cxx::private::Result {
    if let Some(err) = err.downcast_ref::<alloc::string::String>() {
        return ::cxx::private::Result::error(std::format!("panic in {label}: {err}"));
    }
    ::cxx::private::Result::error(std::format!("panic in {label}"))
}
