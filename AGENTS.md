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
- Binary output:
  - Code generators:
    - `bazel-bin/cxxbridge` - C++ code generator (executable)
    - `bazel-bin/libcxxbridge_macro-....so` - Rust code generator (proc macro)
  - Libraries:
    - `bazel-bin/libcxx-....rlib` - Core cxx-rs functionality
    - `bazel-bin/kj-rs/libkj_rs-....rlib` - libkj interoperability

### Just Commands (recommended for development)

- `just build` or `just b` - Build the project
- `just test` or `just t` - Run all tests
- `just format` or `just f` - Format code (uses clang-format + rustfmt)
- `just clippy` - Run Rust clippy linter - (e.g., `just clippy kj-rs`)
- `just asan` - Build the project with ASAN
- `just expand` - Expand the cxx::bridge proc macro generated code in kj-rs tests for debugging

## Testing

### Test Types

- **Rust unit tests**: Located in `cfg` mods in .rs implementation files, per standard Rust convention
- **Rust integration tests**: Located in `test_*.rs` files, primarily in `kj-rs/tests/`
- **C++ unit tests**: Located in `*-test.c++` files, primarily in `kj-rs/tests/`

## Architecture

### Dependencies

- **Cap'n Proto source code** available in `external/+capnp_cpp+capnp-cpp` - contains KJ C++ base library and capnproto RPC library. Consult it for all questions about `kj/` and `capnproto/` includes and `kj::` and `capnp::` namespaces.

## C++ Coding Conventions

- **Prefer KJ primitives over std primitives.** This codebase is built on libkj. Use `kj::Maybe<T>` instead of `std::optional<T>`, `kj::Own<T>` instead of `std::unique_ptr<T>`, `kj::Vector<T>` instead of `std::vector<T>`, `kj::String`/`kj::StringPtr` instead of `std::string`/`std::string_view`, etc. Only use std types when required by an external interface (e.g., CXX bridge `UniquePtr`).