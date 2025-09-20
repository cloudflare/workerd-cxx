//! This crate provides ffi layer to `kj/async-io.h` classes.
//!
//! Currently it includes `AsyncInputStream`, `AsyncOutputStream` and `AsyncIoStream` traits and
//! corresponding bridges to implement and use `kj::AsyncInputStream`, `kj::AsyncOutputStream` and
//! `kj::AsyncIoStream` from Rust.
pub mod ffi;
pub mod r#impl;
mod interface;

pub use interface::{
    AncillaryMessage, AncillaryMessageHandler, AsyncInputStream, AsyncIoStream, AsyncOutputStream,
};

pub type Result<T> = std::io::Result<T>;
