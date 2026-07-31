Important changes:

- `no-abort` replacing panic with exception handling. Every ffi function reports failure
    through its return value in both directions: an `extern "Rust"` function which panics
    reports a `kj::Exception` to C++, and an `extern "C++"` function which throws reports
    the exception to Rust, which panics if the signature has no `Result`. Neither
    direction is allowed to terminate the process, so no ffi shim is truly `noexcept` and
    every return value travels through an out parameter.
- `require lifetime annotations on all returned references` - `no-abort` dependency

Misc changes:
- `__WORKERD_CXX__ define to identify when workerd-cxx fork is in use` crate names are still
    `cxx*`, so extra mechanism is needed to identify the presence of the fork.
- `build fixes` to better suit our build environment. These might be obsolete since there seems to 
    better rules_rust support upstream.
- `site fixes` fixing CI not to build a site and to run periodically
