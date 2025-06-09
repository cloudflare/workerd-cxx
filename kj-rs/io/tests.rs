//! Test utilities for the kj-rs io module
//!
//! This module provides mock implementations and test utilities for the async I/O stream
//! implementations, including support for both Rust and C++ tests via FFI.

use futures::executor::block_on;
use std::future::Future;

use io::{AsyncInputStream, AsyncIoStream, AsyncOutputStream, Result};

#[cfg(test)]
use io::unoptimized_pump_to;

// Mock implementations for testing

/// Mock input stream that provides predefined data
pub struct MockInputStream {
    data: Vec<u8>,
    position: usize,
}

impl MockInputStream {
    /// Create a new mock input stream with the given data
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }

    /// Get the remaining bytes in the stream
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }
}

impl AsyncInputStream for MockInputStream {
    async fn try_read(
        &mut self,
        buffer: &mut [u8],
        _min_bytes: usize,
    ) -> Result<usize> {
        let available = self.data.len() - self.position;
        let to_read = std::cmp::min(buffer.len(), available);

        if to_read == 0 {
            return Ok(0); // EOF
        }

        buffer[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
        self.position += to_read;
        Ok(to_read)
    }

    fn try_get_length(&self) -> Option<u64> {
        Some((self.data.len() - self.position) as u64)
    }
}

/// Mock output stream that captures written data
pub struct MockOutputStream {
    data: Vec<u8>,
    write_error: bool,
    disconnected: bool,
}

impl MockOutputStream {
    /// Create a new mock output stream
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            write_error: false,
            disconnected: false,
        }
    }

    /// Configure the stream to return write errors
    #[must_use]
    pub fn with_error(mut self) -> Self {
        self.write_error = true;
        self
    }

    /// Configure the stream as disconnected
    #[must_use]
    pub fn with_disconnected(mut self) -> Self {
        self.disconnected = true;
        self
    }

    /// Get the data that has been written to the stream
    #[must_use]
    pub fn written_data(&self) -> &[u8] {
        &self.data
    }
}

impl Default for MockOutputStream {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncOutputStream for MockOutputStream {
    async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        if self.write_error {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock write error",
            ));
        }
        self.data.extend_from_slice(buffer);
        Ok(())
    }

    async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()> {
        for piece in pieces {
            self.write(piece).await?;
        }
        Ok(())
    }

    async fn when_write_disconnected(&mut self) -> Result<()> {
        if self.disconnected {
            Ok(())
        } else {
            // Simulate never resolving for non-disconnected streams
            std::future::pending().await
        }
    }
}

/// Mock bidirectional I/O stream
pub struct MockIoStream {
    input: MockInputStream,
    output: MockOutputStream,
    shutdown: bool,
}

impl MockIoStream {
    /// Create a new mock I/O stream with the given input data
    #[must_use]
    pub fn new(input_data: Vec<u8>) -> Self {
        Self {
            input: MockInputStream::new(input_data),
            output: MockOutputStream::new(),
            shutdown: false,
        }
    }

    /// Get the data that has been written to the output side
    #[must_use]
    pub fn written_data(&self) -> &[u8] {
        self.output.written_data()
    }

    /// Check if the stream has been shut down for writing
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }
}

impl AsyncInputStream for MockIoStream {
    async fn try_read(
        &mut self,
        buffer: &mut [u8],
        min_bytes: usize,
    ) -> Result<usize> {
        self.input.try_read(buffer, min_bytes).await
    }

    fn try_get_length(&self) -> Option<u64> {
        self.input.try_get_length()
    }
}

impl AsyncOutputStream for MockIoStream {
    async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        self.output.write(buffer).await
    }

    async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()> {
        self.output.write_vectored(pieces).await
    }

    async fn when_write_disconnected(&mut self) -> Result<()> {
        self.output.when_write_disconnected().await
    }
}

impl AsyncIoStream for MockIoStream {
    async fn shutdown_write(&mut self) -> Result<()> {
        self.shutdown = true;
        Ok(())
    }

    fn abort_read(&mut self) {
        // For mock implementation, we just mark as shutdown
        self.shutdown = true;
    }
}

/// Helper function to run async tests
pub fn run_async_test<F, Fut>(test: F) -> Fut::Output
where
    F: FnOnce() -> Fut,
    Fut: Future,
{
    block_on(test())
}

/// Compute a simple FNV-1a hash of data from an `AsyncInputStream`
/// This is a non-cryptographic hash function suitable for testing
/// 
/// # Errors
/// 
/// Returns an error if reading from the stream fails.
pub async fn compute_stream_hash<T: AsyncInputStream>(stream: &mut T) -> Result<u64> {
    const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;

    let mut hash = FNV_OFFSET_BASIS;
    let mut buffer = [0u8; 4096];

    loop {
        #[allow(unused_variables)]
        let buffer_len = buffer.len();
        let bytes_read = stream.try_read(&mut buffer, 1).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        // FNV-1a hash algorithm
        for &byte in &buffer[..bytes_read] {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }

    Ok(hash)
}

// FFI bridge to expose functionality to C++
#[cxx::bridge(namespace = "kj_rs_io_test")]
pub mod ffi {
    #[namespace = "kj_rs_io"]
    unsafe extern "C++" {
        include!("kj-rs/io/bridge.h");

        type CxxAsyncInputStream = io::ffi::CxxAsyncInputStream;
    }

    extern "Rust" {
        /// Compute hash of data from a CxxAsyncInputStream
        async unsafe fn compute_stream_hash_ffi<'a>(
            stream: UniquePtr<CxxAsyncInputStream>,
        ) -> Result<u64>;
    }
}

/// FFI wrapper function that computes hash from a `CxxAsyncInputStream` FFI object
/// 
/// # Safety
/// 
/// This function is marked as unsafe because it's called from C++ via FFI bridge.
/// The caller must ensure the stream pointer is valid for the lifetime of the call.
/// 
/// # Errors
/// 
/// Returns an error if reading from the stream fails or if C++ exceptions occur.
#[allow(clippy::needless_lifetimes)]
pub async unsafe fn compute_stream_hash_ffi<'a>(
    stream: UniquePtr<ffi::CxxAsyncInputStream>,
) -> Result<u64> {
    let mut wrapper = io::CxxAsyncInputStream::new(stream);
    compute_stream_hash(&mut wrapper).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for AsyncInputStream trait

    #[test]
    fn test_mock_input_stream_basic_read() {
        run_async_test(|| async {
            let mut stream = MockInputStream::new(b"Hello, World!".to_vec());
            let mut buffer = [0u8; 5];

            let bytes_read = stream.try_read(&mut buffer, 1).await.unwrap();
            assert_eq!(bytes_read, 5);
            assert_eq!(&buffer, b"Hello");
            assert_eq!(stream.remaining(), 8);
        });
    }

    #[test]
    fn test_mock_input_stream_try_get_length() {
        let stream = MockInputStream::new(b"Hello".to_vec());
        assert_eq!(stream.try_get_length(), Some(5));
    }

    // Tests for AsyncOutputStream trait

    #[test]
    fn test_mock_output_stream_basic_write() {
        run_async_test(|| async {
            let mut stream = MockOutputStream::new();

            stream.write(b"Hello").await.unwrap();
            stream.write(b", World!").await.unwrap();

            assert_eq!(stream.written_data(), b"Hello, World!");
        });
    }

    #[test]
    fn test_mock_output_stream_write_vectored() {
        run_async_test(|| async {
            let mut stream = MockOutputStream::new();
            let pieces = [b"Hello".as_slice(), b", ".as_slice(), b"World!".as_slice()];

            stream.write_vectored(&pieces).await.unwrap();

            assert_eq!(stream.written_data(), b"Hello, World!");
        });
    }

    #[test]
    fn test_mock_output_stream_write_error() {
        run_async_test(|| async {
            let mut stream = MockOutputStream::new().with_error();

            let result = stream.write(b"test").await;
            assert!(result.is_err());
            assert_eq!(stream.written_data().len(), 0);
        });
    }

    #[test]
    fn test_mock_output_stream_when_write_disconnected() {
        run_async_test(|| async {
            let mut stream = MockOutputStream::new().with_disconnected();

            // This should complete immediately for disconnected stream
            let result = stream.when_write_disconnected().await;
            assert!(result.is_ok());
        });
    }

    // Tests for AsyncIoStream trait

    #[test]
    fn test_mock_io_stream_bidirectional() {
        run_async_test(|| async {
            let mut stream = MockIoStream::new(b"input data".to_vec());

            // Test reading
            let mut buffer = [0u8; 5];
            let bytes_read = stream.try_read(&mut buffer, 1).await.unwrap();
            assert_eq!(bytes_read, 5);
            assert_eq!(&buffer, b"input");

            // Test writing
            stream.write(b"output").await.unwrap();
            assert_eq!(stream.written_data(), b"output");
        });
    }

    #[test]
    fn test_mock_io_stream_shutdown() {
        run_async_test(|| async {
            let mut stream = MockIoStream::new(b"test".to_vec());

            assert!(!stream.is_shutdown());
            stream.shutdown_write().await.unwrap();
            assert!(stream.is_shutdown());
        });
    }

    #[test]
    fn test_mock_io_stream_abort_read() {
        let mut stream = MockIoStream::new(b"test".to_vec());

        assert!(!stream.is_shutdown());
        stream.abort_read();
        assert!(stream.is_shutdown());
    }

    // Tests for utility functions

    #[test]
    fn test_compute_stream_hash() {
        run_async_test(|| async {
            let mut stream = MockInputStream::new(b"Hello, World!".to_vec());
            let hash = compute_stream_hash(&mut stream).await.unwrap();

            // Hash should be deterministic
            assert!(hash > 0);

            // Test with same data again
            let mut stream2 = MockInputStream::new(b"Hello, World!".to_vec());
            let hash2 = compute_stream_hash(&mut stream2).await.unwrap();
            assert_eq!(hash, hash2);
        });
    }

    #[test]
    fn test_compute_stream_hash_different_data() {
        run_async_test(|| async {
            let mut stream1 = MockInputStream::new(b"Hello, World!".to_vec());
            let mut stream2 = MockInputStream::new(b"Hello, Rust!".to_vec());

            let hash1 = compute_stream_hash(&mut stream1).await.unwrap();
            let hash2 = compute_stream_hash(&mut stream2).await.unwrap();

            // Different data should produce different hashes
            assert_ne!(hash1, hash2);
        });
    }

    #[test]
    fn test_compute_stream_hash_empty() {
        const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
        
        run_async_test(|| async {
            let mut stream = MockInputStream::new(Vec::new());
            let hash = compute_stream_hash(&mut stream).await.unwrap();

            // Empty data should produce the FNV offset basis
            assert_eq!(hash, FNV_OFFSET_BASIS);
        });
    }

    #[test]
    fn test_unoptimized_pump_to() {
        run_async_test(|| async {
            let mut input = MockInputStream::new(b"pump test data".to_vec());
            let mut output = MockOutputStream::new();

            let bytes_pumped = unoptimized_pump_to(&mut input, &mut output, 1000, 0)
                .await
                .unwrap();

            assert_eq!(bytes_pumped, 14);
            assert_eq!(output.written_data(), b"pump test data");
            assert_eq!(input.remaining(), 0);
        });
    }

    #[test]
    fn test_unoptimized_pump_to_partial() {
        run_async_test(|| async {
            let mut input = MockInputStream::new(b"pump test data".to_vec());
            let mut output = MockOutputStream::new();

            // Only pump 4 bytes
            let bytes_pumped = unoptimized_pump_to(&mut input, &mut output, 4, 0)
                .await
                .unwrap();

            assert_eq!(bytes_pumped, 4);
            assert_eq!(output.written_data(), b"pump");
            assert_eq!(input.remaining(), 10);
        });
    }

    #[test]
    fn test_unoptimized_pump_to_with_completed() {
        run_async_test(|| async {
            let mut input = MockInputStream::new(b"test".to_vec());
            let mut output = MockOutputStream::new();

            // Start with 2 bytes already completed
            let bytes_pumped = unoptimized_pump_to(&mut input, &mut output, 6, 2)
                .await
                .unwrap();

            assert_eq!(bytes_pumped, 6); // 4 new + 2 already completed
            assert_eq!(output.written_data(), b"test");
        });
    }

    // Integration tests that would work with actual FFI implementations
    // Note: These are placeholders since we don't have actual C++ stream implementations available in tests

    #[test]
    fn test_trait_interfaces_exist() {
        // Test that our trait types exist and have the right interfaces
        fn _check_input_stream_interface<T: AsyncInputStream>(_stream: T) {}
        fn _check_output_stream_interface<T: AsyncOutputStream>(_stream: T) {}
        fn _check_io_stream_interface<T: AsyncIoStream>(_stream: T) {}

        // This test exists mainly to ensure the traits compile correctly
        // The actual FFI wrapper types would be tested with C++ integration tests
    }

    #[cfg(test)]
    mod extension_trait_tests {
        use super::*;
        use io::{AsyncInputStreamExt, AsyncOutputStreamExt};

        #[test]
        fn test_async_read_adapter() {
            let stream = MockInputStream::new(b"test".to_vec());
            let _adapter = stream.into_async_read();
            // Test that the adapter can be created
        }

        #[test]
        fn test_async_write_adapter() {
            let stream = MockOutputStream::new();
            let _adapter = stream.into_async_write();
            // Test that the adapter can be created
        }
    }
}
