//! Test utilities for the kj-rs io module
//!
//! This module provides mock implementations and test utilities for the async I/O stream
//! implementations, including support for both Rust and C++ tests via FFI.

use async_trait::async_trait;
use futures::executor::block_on;
use std::{future::Future, pin::Pin};

use io::ffi::RustAsyncInputStream;
use io::{AsyncInputStream, AsyncIoStream, AsyncOutputStream, Result};

// Mock implementations for testing

/// Mock input stream that provides predefined data
pub struct MockInputStream {
    data: Vec<u8>,
    position: usize,
    read_error: bool,
    known_size: bool,
}

impl MockInputStream {
    /// Create a new mock input stream with the given data
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            position: 0,
            read_error: false,
            known_size: true,
        }
    }

    /// Configure the stream to return read errors
    #[must_use]
    pub fn with_error(mut self) -> Self {
        self.read_error = true;
        self
    }

    /// Configure the stream to not report its length
    #[must_use]
    pub fn with_unknown_length(mut self) -> Self {
        self.known_size = false;
        self
    }

    /// Get the remaining bytes in the stream
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }

    /// Unsafe version of `try_read` that directly calls the `AsyncInputStream` implementation
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the stream fails, such as when the stream is configured to return errors.
    pub async fn try_read_unsafe(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize> {
        <MockInputStream as AsyncInputStream>::try_read(self, buffer, min_bytes).await
    }
}

#[async_trait(?Send)]
impl AsyncInputStream for MockInputStream {
    async fn try_read(&mut self, buffer: &mut [u8], _min_bytes: usize) -> Result<usize> {
        if self.read_error {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock read error",
            ));
        }

        let available = self.data.len() - self.position;
        let to_read = std::cmp::min(buffer.len(), available);

        if to_read == 0 {
            return Ok(0); // EOF
        }

        buffer[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
        self.position += to_read;
        Ok(to_read)
    }

    fn try_get_length(&self) -> Option<usize> {
        if self.known_size {
            Some(self.data.len() - self.position)
        } else {
            None
        }
    }

    async fn pump_to(
        &mut self,
        output: &mut dyn AsyncOutputStream,
        amount: usize,
    ) -> Result<usize> {
        let mut buffer = [0u8; 4096];
        let mut total_pumped = 0;
        let mut remaining = amount;

        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buffer.len());
            let bytes_read = self.try_read(&mut buffer[..to_read], 1).await?;

            if bytes_read == 0 {
                break; // EOF
            }

            output.write(&buffer[..bytes_read]).await?;
            total_pumped += bytes_read;
            remaining = remaining.saturating_sub(bytes_read);
        }

        Ok(total_pumped)
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

#[async_trait(?Send)]
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

#[async_trait(?Send)]
impl AsyncInputStream for MockIoStream {
    async fn try_read(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize> {
        <MockInputStream as AsyncInputStream>::try_read(&mut self.input, buffer, min_bytes).await
    }

    fn try_get_length(&self) -> Option<usize> {
        self.input.try_get_length()
    }

    async fn pump_to(
        &mut self,
        output: &mut dyn AsyncOutputStream,
        amount: usize,
    ) -> Result<usize> {
        self.input.pump_to(output, amount).await
    }
}

#[async_trait(?Send)]
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

#[async_trait(?Send)]
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

/// Generate pseudorandom data to an `AsyncOutputStream` in 1024-byte chunks
///
/// # Errors
///
/// Returns an error if writing to the stream fails.
pub async fn generate_prng<T: AsyncOutputStream>(stream: &mut T, size: u64) -> Result<()> {
    const CHUNK_SIZE: usize = 1024;
    let mut prng = 0u8;
    let mut remaining = size;
    let mut buffer = [0u8; CHUNK_SIZE];

    while remaining > 0 {
        #[allow(clippy::cast_possible_truncation)]
        let chunk_size = std::cmp::min(remaining as usize, CHUNK_SIZE);

        // Fill buffer with pseudorandom data
        for item in buffer.iter_mut().take(chunk_size) {
            prng = prng.wrapping_mul(5).wrapping_add(1);
            *item = prng;
        }

        // Write the chunk
        stream.write(&buffer[..chunk_size]).await?;
        remaining -= chunk_size as u64;
    }

    Ok(())
}

// FFI bridge to expose functionality to C++
#[cxx::bridge(namespace = "kj_rs_io_test")]
pub mod ffi {
    #[namespace = "kj_rs_io::ffi"]
    unsafe extern "C++" {
        include!("kj-rs/io/bridge.h");
        include!("kj-rs/io/ffi.rs.h");

        type CxxAsyncInputStream = io::ffi::bridge::CxxAsyncInputStream;
        type CxxAsyncOutputStream = io::ffi::bridge::CxxAsyncOutputStream;
        type CxxAsyncIoStream = io::ffi::bridge::CxxAsyncIoStream;
    }

    extern "Rust" {
        #[namespace = "kj_rs_io::ffi"]
        type RustAsyncInputStream = io::ffi::RustAsyncInputStream;

        /// Compute hash of data from a `CxxAsyncInputStream`
        async unsafe fn compute_stream_hash_ffi<'a>(
            stream: Pin<&'a mut CxxAsyncInputStream>,
        ) -> Result<u64>;

        /// Generate pseudorandom data to a `CxxAsyncOutputStream`
        async unsafe fn generate_prng_ffi<'a>(
            stream: Pin<&'a mut CxxAsyncOutputStream>,
            size: u64,
        ) -> Result<()>;

        /// Create a Rust MockInputStream with given data that reports its length
        #[allow(clippy::unnecessary_box_returns)]
        fn mock_input_stream(data: &[u8]) -> Box<RustAsyncInputStream>;

        /// Create a Rust MockInputStream with given data that doesn't report its length
        #[allow(clippy::unnecessary_box_returns)]
        fn mock_input_stream_no_size(data: &[u8]) -> Box<RustAsyncInputStream>;

        /// Create a Rust MockInputStream that returns read errors
        #[allow(clippy::unnecessary_box_returns)]
        fn mock_input_stream_with_error() -> Box<RustAsyncInputStream>;

        // /// Create a Rust MockIoStream with given input data
        // fn create_rust_mock_io_stream(input_data: &[u8]) -> Box<RustAsyncIoStream>;

        // /// Compute hash of data from a `RustAsyncInputStream`
        // async unsafe fn compute_stream_hash_rust_ffi<'a>(
        //     stream: &'a mut RustAsyncInputStream,
        // ) -> Result<u64>;

        // /// Compute hash of data from the input side of a `RustAsyncIoStream`
        // async unsafe fn compute_stream_hash_iostream_ffi<'a>(
        //     stream: &'a mut RustAsyncIoStream,
        // ) -> Result<u64>;
    }
}

/// Compute hash of data from a `CxxAsyncInputStream`
///
/// # Safety
///
/// The caller must ensure that the stream pointer is valid and properly aligned.
///
/// # Errors
///
/// Returns an error if reading from the stream fails.
#[allow(clippy::needless_lifetimes)]
pub async unsafe fn compute_stream_hash_ffi<'a>(
    stream: Pin<&'a mut ffi::CxxAsyncInputStream>,
) -> Result<u64> {
    let mut wrapper = io::ffi::CxxAsyncInputStream::new(stream);
    compute_stream_hash(&mut wrapper).await
}

/// Generate pseudorandom data to a `CxxAsyncOutputStream`
///
/// # Safety
///
/// The caller must ensure that the stream pointer is valid and properly aligned.
///
/// # Errors
///
/// Returns an error if writing to the stream fails.
#[allow(clippy::needless_lifetimes)]
pub async unsafe fn generate_prng_ffi<'a>(
    stream: Pin<&'a mut ffi::CxxAsyncOutputStream>,
    size: u64,
) -> Result<()> {
    let mut wrapper = io::ffi::CxxAsyncOutputStream::new(stream);
    generate_prng(&mut wrapper, size).await
}

/// Create a Rust `MockInputStream` with given data that reports its length
#[must_use]
pub fn mock_input_stream(data: &[u8]) -> Box<RustAsyncInputStream> {
    Box::new(RustAsyncInputStream::new(MockInputStream::new(
        data.to_vec(),
    )))
}

/// Create a Rust `MockInputStream` with given data that doesn't report its length
#[must_use]
pub fn mock_input_stream_no_size(data: &[u8]) -> Box<RustAsyncInputStream> {
    Box::new(RustAsyncInputStream::new(
        MockInputStream::new(data.to_vec()).with_unknown_length(),
    ))
}

/// Create a Rust `MockOutputStream`
#[must_use]
pub fn create_rust_mock_output_stream() -> Box<io::ffi::RustAsyncOutputStream> {
    let mock_stream = MockOutputStream::new();
    Box::new(io::ffi::RustAsyncOutputStream::new(mock_stream))
}

/// Create a Rust `MockOutputStream` that returns write errors
#[must_use]
pub fn create_rust_mock_output_stream_with_error() -> Box<io::ffi::RustAsyncOutputStream> {
    let mock_stream = MockOutputStream::new().with_error();
    Box::new(io::ffi::RustAsyncOutputStream::new(mock_stream))
}

/// Create a Rust `MockIoStream` with given input data
#[must_use]
pub fn create_rust_mock_io_stream(input_data: &[u8]) -> Box<io::ffi::RustAsyncIoStream> {
    let mock_stream = MockIoStream::new(input_data.to_vec());
    Box::new(io::ffi::RustAsyncIoStream::new(mock_stream))
}

/// Create a Rust `MockInputStream` that returns read errors
#[must_use]
pub fn mock_input_stream_with_error() -> Box<io::ffi::RustAsyncInputStream> {
    let mock_stream = MockInputStream::new(Vec::new()).with_error();
    Box::new(io::ffi::RustAsyncInputStream::new(mock_stream))
}

/// Get the data written to a `MockOutputStream` (for testing)
///
/// # Panics
///
/// This function will panic if the stream is not actually a `MockOutputStream`.
/// This is intended for testing only.
#[must_use]
pub fn get_mock_output_stream_data(_stream: &io::ffi::RustAsyncOutputStream) -> Vec<u8> {
    // Note: This is a bit of a hack since we can't easily downcast through the FFI boundary.
    // For real tests, we'd need a better way to extract test data.
    // For now, this serves as a placeholder for the interface.
    Vec::new()
}

/// Compute hash of data from a `RustAsyncInputStream`
///
/// # Safety
///
/// The caller must ensure that the stream pointer is valid and properly aligned.
///
/// # Errors
///
/// Returns an error if reading from the stream fails.
pub async unsafe fn compute_stream_hash_rust_ffi(
    stream: &mut io::ffi::RustAsyncInputStream,
) -> Result<u64> {
    const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;

    let mut hash = FNV_OFFSET_BASIS;
    let mut buffer = [0u8; 4096];

    loop {
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

/// Compute hash of data from the input side of a `RustAsyncIoStream`
///
/// # Safety
///
/// The caller must ensure that the stream pointer is valid and properly aligned.
///
/// # Errors
///
/// Returns an error if reading from the stream fails.
pub async unsafe fn compute_stream_hash_iostream_ffi(
    stream: &mut io::ffi::RustAsyncIoStream,
) -> Result<u64> {
    const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;

    let mut hash = FNV_OFFSET_BASIS;
    let mut buffer = [0u8; 4096];

    loop {
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

#[cfg(test)]
mod tests {
    use io::r#impl::unoptimized_pump_to;

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

    #[test]
    fn test_generate_prng() {
        run_async_test(|| async {
            let mut output = MockOutputStream::new();

            // Generate 100 bytes of pseudorandom data
            generate_prng(&mut output, 100).await.unwrap();

            let data = output.written_data();
            assert_eq!(data.len(), 100);

            // Test that the data is deterministic
            let mut output2 = MockOutputStream::new();
            generate_prng(&mut output2, 100).await.unwrap();
            assert_eq!(data, output2.written_data());
        });
    }

    #[test]
    fn test_generate_prng_large() {
        run_async_test(|| async {
            let mut output = MockOutputStream::new();

            // Generate 2048 bytes (multiple chunks)
            generate_prng(&mut output, 2048).await.unwrap();

            let data = output.written_data();
            assert_eq!(data.len(), 2048);

            // Check that the data varies (basic test that generator is working)
            #[allow(clippy::naive_bytecount)]
            let zero_count = data.iter().filter(|&&b| b == 0).count();
            assert!(zero_count < data.len() / 10); // Less than 10% zeros
        });
    }

    #[test]
    fn test_generate_prng_partial_chunk() {
        run_async_test(|| async {
            let mut output = MockOutputStream::new();

            // Generate 1500 bytes (1 full chunk + partial chunk)
            generate_prng(&mut output, 1500).await.unwrap();

            let data = output.written_data();
            assert_eq!(data.len(), 1500);
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
}
