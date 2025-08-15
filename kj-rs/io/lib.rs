//! This crate provides ffi layer to `kj/async-io.h` classes.
//!
//! Currently it includes `AsyncInputStream`, `AsyncOutputStream` and `AsyncIoStream` traits.
use std::{future::Future, pin::Pin};

pub mod ffi;
pub mod r#impl;
mod interface;

pub use interface::{
    AncillaryMessage, AncillaryMessageHandler, AsyncInputStream, AsyncIoStream, AsyncOutputStream,
};

pub type Result<T> = std::io::Result<T>;
