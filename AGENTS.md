# AGENTS.md

This file provides guidance to AI agents such as [OpenCode](https://opencode.ai/).

## Instructions for AI Code Assistants

- Suggest updates to AGENTS.md when you find new high-level information.

## Project Overview

**workerd-cxx** is Cloudflare's fork of the [cxx-rs](https://cxx.rs/) Rust crate. Our fork contains modifications to support Rust interoperability with libkj, the core C++ library used by [workerd](https://github.com/cloudflare/workerd).

Since **workerd** is a cross-platform project which targets Linux, macOS, and Windows, **workerd-cxx** also targets Linux, macOS, and Windows.

## Build System & Commands

### Primary Build System: Bazel

- Main build command: `bazel build //...`

### Just Commands (recommended for development)

- `just build` or `just b` - Build the project
- `just test` or `just t` - Run all tests
- `just format` or `just f` - Format code (uses clang-format + rustfmt)
- `just clippy` - Run Rust clippy linter - (e.g., `just clippy kj-rs`)
- `just asan` - Build the project with ASAN
- `just expand` - Expand the cxx::bridge proc macro generated code in kj-rs tests for debugging

### Before Committing

Always run `just clippy` (or `bazel build --config=clippy //...`) before committing Rust changes. CI enforces clippy with pedantic lints including `doc_markdown`, which requires backticks around type names in doc comments.

Always run `just format` before committing to ensure consistent formatting across Rust and C++ files.

## Testing

### Test Types

- **Rust unit tests**: Located in `cfg` mods in .rs implementation files, per standard Rust convention
- **Rust integration tests**: Located in `test_*.rs` files, primarily in `kj-rs/tests/`
- **C++ unit tests**: Located in `*-test.c++` files, primarily in `kj-rs/tests/`

### Async tests

Async tests require a `kj::EventLoop` and are driven from C++ using `KJ_TEST`, not from Rust. See `kj-rs/tests/awaitables-cc-test.c++`. Rust helper functions (in `test_futures.rs`) provide async behaviors that the C++ test driver co_awaits or polls.

## Architecture

### CXX Bridge

FFI declarations live in `#[cxx::bridge]` blocks in `lib.rs` files. The proc macro generates C++ headers (`lib.rs.h`) that C++ code includes. Marking a function `async` in `extern "Rust"` makes it return `kj::Promise<T>` on the C++ side; marking one `async` in `extern "C++"` makes it return `impl Future` on the Rust side.

### Async FFI (kj-rs/)

The `kj-rs/` directory bridges KJ promises and Rust futures:

- **C++ polling Rust futures**: `future.h` (`FutureAwaiter`, `RustFuture`) -- wraps a Rust `dyn Future` as a `kj::Promise<T>`. Dropping the promise cancels the Rust future.
- **Rust awaiting KJ promises**: `awaiter.h`/`awaiter.c++` (`RustPromiseAwaiter`) and `awaiter.rs`/`promise.rs` (`PromiseAwaiter`, `PromiseFuture`) -- wraps a `kj::Promise<T>` as a Rust `impl Future`. Dropping the future cancels the KJ promise.
- **Waker integration**: `waker.h` (`ArcWaker`, `LazyArcWaker`) -- bridges Rust's `Waker` with KJ's `Event` system. The "optimized path" links directly to the `FuturePollEvent` via `LinkedGroup`; the fallback path clones an `ArcWaker`.

Cancellation is implicit in both directions: dropping a `kj::Promise` or Rust `Future` synchronously and recursively cancels the underlying work.

### Dependencies

- **Cap'n Proto source code** available in `external/+capnp_cpp+capnp-cpp` - contains KJ C++ base library and capnproto RPC library. Consult it for all questions about `kj/` and `capnproto/` includes and `kj::` and `capnp::` namespaces.

## C++ Coding Conventions

- **Prefer KJ primitives over std primitives.** This codebase is built on libkj. Use `kj::Maybe<T>` instead of `std::optional<T>`, `kj::Own<T>` instead of `std::unique_ptr<T>`, `kj::Vector<T>` instead of `std::vector<T>`, `kj::String`/`kj::StringPtr` instead of `std::string`/`std::string_view`, etc. Only use std types when required by an external interface (e.g., CXX bridge `UniquePtr`).

## Rust Coding Conventions

- **Error handling**: Async FFI functions return `Result<T>` which translates to `kj::Exception` on the C++ side. Use `cxx::KjError` for direct control over KJ exception type, or any `std::error::Error` impl for automatic conversion. See `workerd`'s `src/rust/AGENTS.md` for the full error handling guide.
- **Safety comments**: All `unsafe` blocks must have a `// Safety:` comment explaining why the invariants are upheld.