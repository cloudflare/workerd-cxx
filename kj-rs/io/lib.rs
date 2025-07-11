use std::{future::Future, pin::Pin};

use async_trait::async_trait;
use cxx::type_id;

pub type Result<T> = std::io::Result<T>;

/// Type alias for ancillary message handlers
pub type AncillaryMessageHandler = Box<dyn FnMut(&[AncillaryMessage]) + 'static>;

/// Asynchronous equivalent of `InputStream`.
///
/// This trait corresponds to the C++ `kj::AsyncInputStream` class and provides
/// asynchronous reading capabilities with KJ promise integration.
#[async_trait(?Send)]
pub trait AsyncInputStream {
    /// Try to read some bytes from the stream without blocking indefinitely.
    ///
    /// Like `read()`, but this method is the primitive that subclasses must implement.
    /// It reads at least `min_bytes` and at most `max_bytes` bytes from the stream.
    ///
    /// This is the core method that all `AsyncInputStream` implementations must provide.
    async fn try_read(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize>;

    /// Get the remaining number of bytes that will be produced by this stream, if known.
    ///
    /// This is used e.g. to fill in the Content-Length header of an HTTP message. If unknown, the
    /// HTTP implementation may need to fall back to Transfer-Encoding: chunked.
    ///
    /// The default implementation always returns None.
    fn try_get_length(&self) -> Option<usize> {
        None
    }

    /// Read `amount` bytes from this stream (or to EOF) and write them to `output`, returning the
    /// total bytes actually pumped (which is only less than `amount` if EOF was reached).
    ///
    /// Override this if your stream type knows how to pump itself to certain kinds of output
    /// streams more efficiently than via the naive approach. You can use dynamic downcasting
    /// to test for stream types you recognize, and if none match, delegate to the default
    /// implementation.
    ///
    /// The default implementation performs a naive pump by allocating a buffer and reading to it /
    /// writing from it in a loop.
    async fn pump_to(&mut self, output: &mut dyn AsyncOutputStream, amount: usize) -> Result<usize>;

    /// Register interest in checking for ancillary messages (aka control messages) when reading.
    ///
    /// The provided callback will be called whenever any are encountered. The messages passed to
    /// the function do not live beyond when function returns.
    /// Only supported on Unix (the default impl throws UNIMPLEMENTED). Most apps will not use this.
    ///
    /// # Errors
    ///
    /// Returns an error if ancillary messages are not supported on this platform.
    fn register_ancillary_message_handler(
        &mut self,
        _handler: AncillaryMessageHandler,
    ) -> Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Ancillary messages not supported",
        ))
    }

    /// Primarily intended as an optimization for the `tee` call.
    ///
    /// Returns an input stream whose state is independent from this one but which will return the exact same
    /// set of bytes read going forward. `limit` is a total limit on the amount of memory, in bytes, which a tee
    /// implementation may use to buffer stream data. An implementation must throw an exception if a read operation
    /// would cause the limit to be exceeded. If `try_tee()` can see that the new limit is impossible to
    /// satisfy, it should return None so that the pessimized path is taken in `new_tee`. This is
    /// likely to arise if `try_tee()` is called twice with different limits on the same stream.
    ///
    /// Note: This method is not async fn compatible and implementations should provide concrete types.
    fn try_tee(&mut self, _limit: usize) {
        todo!("Tee implementation not yet available")
    }
}

/// Asynchronous equivalent of `OutputStream`.
///
/// This trait corresponds to the C++ `kj::AsyncOutputStream` class and provides
/// asynchronous writing capabilities with KJ promise integration.
#[async_trait(?Send)]
pub trait AsyncOutputStream {
    /// Write data to the stream.
    ///
    /// The future completes when all data has been written to the stream.
    async fn write(&mut self, buffer: &[u8]) -> Result<()>;

    /// Write multiple pieces of data to the stream.
    ///
    /// This is equivalent to concatenating all the pieces and calling `write()`, but may be
    /// more efficient for some stream types.
    async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()>;

    /// Implements double-dispatch for `AsyncInputStream::pump_to()`.
    ///
    /// This method should only be called from within an implementation of `pump_to()`.
    ///
    /// This method examines the type of `input` to find optimized ways to pump data from it to this
    /// output stream. If it finds one, it performs the pump. Otherwise, it returns None.
    ///
    /// The default implementation always returns None.
    async fn try_pump_from(
        &mut self,
        _input: &mut dyn AsyncInputStream,
        _amount: usize,
    ) -> Option<Result<usize>> {
        None
    }

    /// Returns a future that resolves when the stream has become disconnected such that new write()s
    /// will fail with a DISCONNECTED exception.
    ///
    /// This is particularly useful, for example, to cancel work early when it is detected that no one will
    /// receive the result.
    ///
    /// Note that not all streams are able to detect this condition without actually performing a
    /// `write()`; such stream implementations may return a future that never resolves. (In particular,
    /// as of this writing, `when_write_disconnected()` is not implemented on Windows. Also, for TCP
    /// streams, not all disconnects are detectable -- a power or network failure may lead the
    /// connection to hang forever, or until configured socket options lead to a timeout.)
    ///
    /// Unlike most other asynchronous stream methods, it is safe to call `when_write_disconnected()`
    /// multiple times without canceling the previous futures.
    async fn when_write_disconnected(&mut self) -> Result<()>;
}

/// A combination input and output stream.
///
/// This trait corresponds to the C++ `kj::AsyncIoStream` class and combines both
/// `AsyncInputStream` and `AsyncOutputStream` functionality.
#[async_trait(?Send)]
pub trait AsyncIoStream: AsyncInputStream + AsyncOutputStream {
    /// Cleanly shut down just the write end of the stream, while keeping the read end open.
    async fn shutdown_write(&mut self) -> Result<()>;

    /// Similar to `shutdown_write`, but this will shut down the read end of the stream, and should only
    /// be called when an error has occurred.
    fn abort_read(&mut self) {}

    /// Corresponds to `getsockopt()` syscall.
    ///
    /// Will return an error if the stream is not a socket or the option is not appropriate for the socket type.
    /// The default implementation always returns an "unimplemented" error.
    ///
    /// # Errors
    ///
    /// Returns an error if the stream is not a socket or the option is not supported.
    fn getsockopt(&self, _level: i32, _option: i32, _value: &mut [u8]) -> Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "getsockopt not supported",
        ))
    }

    /// Corresponds to `setsockopt()` syscall.
    ///
    /// Will return an error if the stream is not a socket or the option is not appropriate for the socket type.
    /// The default implementation always returns an "unimplemented" error.
    ///
    /// # Errors
    ///
    /// Returns an error if the stream is not a socket or the option is not supported.
    fn setsockopt(&mut self, _level: i32, _option: i32, _value: &[u8]) -> Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "setsockopt not supported",
        ))
    }

    /// Corresponds to `getsockname()` syscall.
    ///
    /// Will return an error if the stream is not a socket.
    /// The default implementation always returns an "unimplemented" error.
    ///
    /// # Errors
    ///
    /// Returns an error if the stream is not a socket.
    fn getsockname(&self, _addr: &mut [u8]) -> Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "getsockname not supported",
        ))
    }

    /// Corresponds to `getpeername()` syscall.
    ///
    /// Will return an error if the stream is not a socket.
    /// The default implementation always returns an "unimplemented" error.
    ///
    /// # Errors
    ///
    /// Returns an error if the stream is not a socket.
    fn getpeername(&self, _addr: &mut [u8]) -> Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "getpeername not supported",
        ))
    }

    /// Get the underlying Unix file descriptor, if any.
    ///
    /// Returns None if this object actually isn't wrapping a file descriptor.
    fn get_fd(&self) -> Option<i32> {
        None
    }

    /// Get the underlying Win32 HANDLE, if any.
    ///
    /// Returns None if this object actually isn't wrapping a handle.
    fn get_win32_handle(&self) -> Option<*mut std::ffi::c_void> {
        None
    }
}

/// Represents an ancillary message (aka control message) received using the `recvmsg()` system
/// call (or equivalent). Most apps will not use this.
#[derive(Debug, Clone)]
pub struct AncillaryMessage {
    /// Originating protocol / socket level.
    pub level: i32,
    /// Protocol-specific message type.
    pub message_type: i32,
    /// Message data. In most cases you should use the accessor methods.
    pub data: Vec<u8>,
}

impl AncillaryMessage {
    /// Create a new ancillary message.
    #[must_use]
    pub fn new(level: i32, message_type: i32, data: Vec<u8>) -> Self {
        Self {
            level,
            message_type,
            data,
        }
    }

    /// Get the originating protocol / socket level.
    #[must_use]
    pub fn level(&self) -> i32 {
        self.level
    }

    /// Get the protocol-specific message type.
    #[must_use]
    pub fn message_type(&self) -> i32 {
        self.message_type
    }
}

/// Performs a pump using `read()` and `write()`, without calling the stream's `pump_to()` nor
/// `try_pump_from()` methods.
///
/// This is intended to be used as a fallback by implementations of `pump_to()`
/// and `try_pump_from()` when they want to give up on optimization, but can't just call `pump_to()` again
/// because this would recursively retry the optimization. `unoptimized_pump_to()` should only be called
/// inside implementations of streams, never by the caller of a stream -- use the `pump_to()` method
/// instead.
///
/// `completed_so_far` is the number of bytes out of `amount` that have already been pumped. This is
/// provided for convenience for cases where the caller has already done some pumping before they
/// give up. Otherwise, a `.then()` would need to be used to add the bytes to the final result.
///
/// # Errors
///
/// Returns an error if reading from the input stream or writing to the output stream fails.
#[allow(clippy::cast_possible_truncation)]
pub async fn unoptimized_pump_to<I: AsyncInputStream, O: AsyncOutputStream>(
    input: &mut I,
    output: &mut O,
    amount: usize,
    completed_so_far: usize,
) -> Result<usize> {
    let mut buffer = [0u8; 4096];
    let mut total_pumped = completed_so_far;
    let mut remaining = amount.saturating_sub(completed_so_far);

    while remaining > 0 {
        let to_read = std::cmp::min(remaining, buffer.len());
        let bytes_read = input.try_read(&mut buffer[..to_read], 1).await?;

        if bytes_read == 0 {
            break; // EOF
        }

        output.write(&buffer[..bytes_read]).await?;
        total_pumped += bytes_read;
        remaining = remaining.saturating_sub(bytes_read);
    }

    Ok(total_pumped)
}

// Extension traits to provide futures compatibility
pub trait AsyncInputStreamExt: AsyncInputStream {
    /// Convert this stream to implement `futures::io::AsyncRead`
    fn into_async_read(self) -> AsyncReadAdapter<Self>
    where
        Self: Sized + Unpin,
    {
        AsyncReadAdapter(self)
    }
}

pub trait AsyncOutputStreamExt: AsyncOutputStream {
    /// Convert this stream to implement `futures::io::AsyncWrite`
    fn into_async_write(self) -> AsyncWriteAdapter<Self>
    where
        Self: Sized + Unpin,
    {
        AsyncWriteAdapter(self)
    }
}

// Implement the extension traits for all implementations
impl<T: AsyncInputStream> AsyncInputStreamExt for T {}
impl<T: AsyncOutputStream> AsyncOutputStreamExt for T {}

/// Adapter to implement `futures::io::AsyncRead` for `AsyncInputStream`
pub struct AsyncReadAdapter<T>(T);

impl<T> AsyncReadAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn get_ref(&self) -> &T {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// Adapter to implement `futures::io::AsyncWrite` for `AsyncOutputStream`
pub struct AsyncWriteAdapter<T>(T);

impl<T> AsyncWriteAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn get_ref(&self) -> &T {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: AsyncInputStream + Unpin> futures::io::AsyncRead for AsyncReadAdapter<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        use std::task::Poll;

        let min_bytes = std::cmp::min(1, buf.len());

        let future = self.0.try_read(buf, min_bytes);
        let mut pinned_future = Box::pin(future);

        match pinned_future.as_mut().poll(cx) {
            Poll::Ready(result) => Poll::Ready(result),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: AsyncOutputStream + Unpin> futures::io::AsyncWrite for AsyncWriteAdapter<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        use std::task::Poll;

        let future = self.0.write(buf);
        let mut pinned_future = Box::pin(future);

        match pinned_future.as_mut().poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // KJ streams don't have explicit flush, so we just return ready
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // KJ streams don't have explicit close, so we just return ready
        std::task::Poll::Ready(Ok(()))
    }
}

#[cxx::bridge(namespace = "kj_rs_io::ffi")]
#[allow(clippy::needless_lifetimes)]
pub mod ffi {

    // Rust opaque types that can be used from C++
    extern "Rust" {
        /// Opaque Rust type implementing AsyncInputStream trait
        type RustAsyncInputStream;

        /// Opaque Rust type implementing AsyncOutputStream trait
        type RustAsyncOutputStream;

        /// Opaque Rust type implementing AsyncIoStream trait
        type RustAsyncIoStream;

        // RustAsyncInputStream methods
        async unsafe fn try_read<'a>(
            self: &'a mut RustAsyncInputStream,
            buffer: &'a mut [u8],
            min_bytes: usize,
        ) -> Result<usize>;

        fn try_get_length(self: &RustAsyncInputStream) -> usize; // Return 0 if unknown

        async unsafe fn pump_to<'a>(
            self: &'a mut RustAsyncInputStream,
            output: &'a mut RustAsyncOutputStream,
            amount: usize,
        ) -> Result<usize>;

        // RustAsyncOutputStream methods
        async unsafe fn write<'a>(
            self: &'a mut RustAsyncOutputStream,
            buffer: &'a [u8],
        ) -> Result<()>;

        async unsafe fn write_vectored<'a>(
            self: &'a mut RustAsyncOutputStream,
            pieces: &'a [&'a [u8]],
        ) -> Result<()>;

        async unsafe fn try_pump_from<'a>(
            self: &'a mut RustAsyncOutputStream,
            input: &'a mut RustAsyncInputStream,
            amount: usize,
        ) -> Result<usize>; // Returns 0 if not supported

        async unsafe fn when_write_disconnected<'a>(
            self: &'a mut RustAsyncOutputStream,
        ) -> Result<()>;

        // RustAsyncIoStream methods - inherited from AsyncInputStream
        async unsafe fn try_read<'a>(
            self: &'a mut RustAsyncIoStream,
            buffer: &'a mut [u8],
            min_bytes: usize,
        ) -> Result<usize>;

        fn try_get_length(self: &RustAsyncIoStream) -> usize;

        async unsafe fn pump_to<'a>(
            self: &'a mut RustAsyncIoStream,
            output: &'a mut RustAsyncOutputStream,
            amount: usize,
        ) -> Result<usize>;

        // RustAsyncIoStream methods - inherited from AsyncOutputStream
        async unsafe fn write<'a>(self: &'a mut RustAsyncIoStream, buffer: &'a [u8]) -> Result<()>;

        async unsafe fn write_vectored<'a>(
            self: &'a mut RustAsyncIoStream,
            pieces: &'a [&'a [u8]],
        ) -> Result<()>;

        async unsafe fn try_pump_from<'a>(
            self: &'a mut RustAsyncIoStream,
            input: &'a mut RustAsyncInputStream,
            amount: usize,
        ) -> Result<usize>;

        async unsafe fn when_write_disconnected<'a>(self: &'a mut RustAsyncIoStream) -> Result<()>;

        // RustAsyncIoStream methods - specific to IoStream
        async unsafe fn shutdown_write<'a>(self: &'a mut RustAsyncIoStream) -> Result<()>;

        fn abort_read(self: &mut RustAsyncIoStream);
    }

    impl Box<RustAsyncInputStream> {}
    impl Box<RustAsyncOutputStream> {}
    impl Box<RustAsyncIoStream> {}

    unsafe extern "C++" {
        include!("kj-rs/io/bridge.h");

        /// Opaque C++ type representing kj::AsyncInputStream
        type CxxAsyncInputStream;

        /// Opaque C++ type representing kj::AsyncOutputStream  
        type CxxAsyncOutputStream;

        /// Opaque C++ type representing kj::AsyncIoStream
        type CxxAsyncIoStream;

        // CxxAsyncInputStream methods
        async fn try_read<'a>(
            self: Pin<&'a mut CxxAsyncInputStream>,
            buffer: &'a mut [u8],
            min_bytes: usize,
        ) -> Result<usize>;

        #[must_use]
        fn try_get_length(self: Pin<&mut CxxAsyncInputStream>) -> usize; // Return 0 if unknown

        async fn pump_to<'a>(
            self: Pin<&'a mut CxxAsyncInputStream>,
            output: Pin<&'a mut CxxAsyncOutputStream>,
            amount: usize,
        ) -> Result<usize>;

        // CxxAsyncOutputStream methods
        async fn write<'a>(self: Pin<&'a mut CxxAsyncOutputStream>, buffer: &'a [u8])
        -> Result<()>;

        async fn write_vectored<'a>(
            self: Pin<&'a mut CxxAsyncOutputStream>,
            pieces: &'a [&'a [u8]],
        ) -> Result<()>;

        async fn try_pump_from<'a>(
            self: Pin<&'a mut CxxAsyncOutputStream>,
            input: Pin<&'a mut CxxAsyncInputStream>,
            amount: usize,
        ) -> Result<usize>; // Returns 0 if not supported

        async fn when_write_disconnected<'a>(self: Pin<&'a mut CxxAsyncOutputStream>)
        -> Result<()>;

        // CxxAsyncIoStream methods - inherited from AsyncInputStream
        async fn try_read<'a>(
            self: Pin<&'a mut CxxAsyncIoStream>,
            buffer: &'a mut [u8],
            min_bytes: usize,
        ) -> Result<usize>;

        #[must_use]
        fn try_get_length(self: Pin<&mut CxxAsyncIoStream>) -> usize;

        async fn pump_to<'a>(
            self: Pin<&'a mut CxxAsyncIoStream>,
            output: Pin<&'a mut CxxAsyncOutputStream>,
            amount: usize,
        ) -> Result<usize>;

        // CxxAsyncIoStream methods - inherited from AsyncOutputStream
        async fn write<'a>(self: Pin<&'a mut CxxAsyncIoStream>, buffer: &'a [u8]) -> Result<()>;

        async fn write_vectored<'a>(
            self: Pin<&'a mut CxxAsyncIoStream>,
            pieces: &'a [&'a [u8]],
        ) -> Result<()>;

        async fn try_pump_from<'a>(
            self: Pin<&'a mut CxxAsyncIoStream>,
            input: Pin<&'a mut CxxAsyncInputStream>,
            amount: usize,
        ) -> Result<usize>;

        async fn when_write_disconnected<'a>(self: Pin<&'a mut CxxAsyncIoStream>) -> Result<()>;

        // CxxAsyncIoStream methods - specific to IoStream
        fn shutdown_write(self: Pin<&mut CxxAsyncIoStream>);

        fn abort_read(self: Pin<&mut CxxAsyncIoStream>);
    }
}

/// Helper function to convert `cxx::Exception` to `std::io::Error`
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn cxx_to_io_error(e: cxx::Exception) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("CXX error: {}", e.what()),
    )
}

/// Rust wrapper for the `CxxAsyncInputStream` FFI type
pub struct CxxAsyncInputStream<'a>(Pin<&'a mut ffi::CxxAsyncInputStream>);

impl<'a> CxxAsyncInputStream<'a> {
    #[must_use]
    pub fn new(inner: Pin<&'a mut ffi::CxxAsyncInputStream>) -> Self {
        Self(inner)
    }
}

#[async_trait(?Send)]
impl AsyncInputStream for CxxAsyncInputStream<'_> {
    async fn try_read(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize> {
        self.0
            .as_mut()
            .try_read(buffer, min_bytes)
            .await
            .map_err(cxx_to_io_error)
    }

    fn try_get_length(&self) -> Option<usize> {
        // Note: This requires a mutable reference to the FFI object, so we can't implement it correctly here
        // For now, return None
        None
    }

    async fn pump_to(&mut self, output: &mut dyn AsyncOutputStream, amount: usize) -> Result<usize> {
        // For now, fall back to the default implementation
        // TODO: Add optimized pump_to for CxxAsyncOutputStream
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

/// Rust wrapper for the `CxxAsyncOutputStream` FFI type  
pub struct CxxAsyncOutputStream<'a>(Pin<&'a mut ffi::CxxAsyncOutputStream>);

impl<'a> CxxAsyncOutputStream<'a> {
    #[must_use]
    pub fn new(inner: Pin<&'a mut ffi::CxxAsyncOutputStream>) -> Self {
        Self(inner)
    }
}

#[async_trait(?Send)]
impl AsyncOutputStream for CxxAsyncOutputStream<'_> {
    async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        self.0.as_mut().write(buffer).await.map_err(cxx_to_io_error)
    }

    async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()> {
        self.0
            .as_mut()
            .write_vectored(pieces)
            .await
            .map_err(cxx_to_io_error)
    }

    async fn try_pump_from(
        &mut self,
        _input: &mut dyn AsyncInputStream,
        _amount: usize,
    ) -> Option<Result<usize>> {
        // For now, return None to indicate no optimization available
        // TODO: Add optimized try_pump_from for CxxAsyncInputStream using trait objects
        None
    }

    async fn when_write_disconnected(&mut self) -> Result<()> {
        self.0
            .as_mut()
            .when_write_disconnected()
            .await
            .map_err(cxx_to_io_error)
    }
}

/// Rust wrapper for the `CxxAsyncIoStream` FFI type
pub struct CxxAsyncIoStream<'a>(Pin<&'a mut ffi::CxxAsyncIoStream>);

impl<'a> CxxAsyncIoStream<'a> {
    #[must_use]
    pub fn new(inner: Pin<&'a mut ffi::CxxAsyncIoStream>) -> Self {
        Self(inner)
    }
}

#[async_trait(?Send)]
impl AsyncInputStream for CxxAsyncIoStream<'_> {
    async fn try_read(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize> {
        self.0
            .as_mut()
            .try_read(buffer, min_bytes)
            .await
            .map_err(cxx_to_io_error)
    }

    fn try_get_length(&self) -> Option<usize> {
        // We need a mutable reference for the FFI call, so we can't implement this correctly
        // For now, return None
        None
    }

    async fn pump_to(&mut self, output: &mut dyn AsyncOutputStream, amount: usize) -> Result<usize> {
        // For now, fall back to the default implementation
        // TODO: Add optimized pump_to for CxxAsyncOutputStream
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

#[async_trait(?Send)]
impl AsyncOutputStream for CxxAsyncIoStream<'_> {
    async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        self.0.as_mut().write(buffer).await.map_err(cxx_to_io_error)
    }

    async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()> {
        self.0
            .as_mut()
            .write_vectored(pieces)
            .await
            .map_err(cxx_to_io_error)
    }

    async fn try_pump_from(
        &mut self,
        _input: &mut dyn AsyncInputStream,
        _amount: usize,
    ) -> Option<Result<usize>> {
        // For now, return None to indicate no optimization available
        // TODO: Add optimized try_pump_from for CxxAsyncInputStream
        None
    }

    async fn when_write_disconnected(&mut self) -> Result<()> {
        self.0
            .as_mut()
            .when_write_disconnected()
            .await
            .map_err(cxx_to_io_error)
    }
}

#[async_trait(?Send)]
impl AsyncIoStream for CxxAsyncIoStream<'_> {
    async fn shutdown_write(&mut self) -> Result<()> {
        self.0.as_mut().shutdown_write();
        Ok(())
    }

    fn abort_read(&mut self) {
        self.0.as_mut().abort_read();
    }
}

// Rust opaque types for use from C++

/// Opaque Rust type that can hold any `AsyncInputStream` implementation
pub struct RustAsyncInputStream(Box<dyn AsyncInputStream>);

impl RustAsyncInputStream {
    pub fn new<T: AsyncInputStream + 'static>(stream: T) -> Self {
        Self(Box::new(stream))
    }

    // FFI method implementations
    /// # Errors
    ///
    /// Returns an error if reading from the stream fails.
    pub async fn try_read(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize> {
        self.0.try_read(buffer, min_bytes).await
    }

    #[must_use]
    pub fn try_get_length(&self) -> usize {
        self.0.try_get_length().unwrap_or(0)
    }

    /// # Errors
    ///
    /// Returns an error if pumping data fails.
    pub async fn pump_to(
        &mut self,
        output: &mut RustAsyncOutputStream,
        amount: usize,
    ) -> Result<usize> {
        self.0.pump_to(output.0.as_mut(), amount).await
    }
}

unsafe impl cxx::ExternType for RustAsyncInputStream {
    type Id = type_id!("kj_rs_io::ffi::RustAsyncInputStream");
    type Kind = cxx::kind::Opaque;
}

/// Opaque Rust type that can hold any `AsyncOutputStream` implementation
pub struct RustAsyncOutputStream(Box<dyn AsyncOutputStream>);

impl RustAsyncOutputStream {
    pub fn new<T: AsyncOutputStream + 'static>(stream: T) -> Self {
        Self(Box::new(stream))
    }

    // FFI method implementations
    /// # Errors
    ///
    /// Returns an error if writing to the stream fails.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        self.0.write(buffer).await
    }

    /// # Errors
    ///
    /// Returns an error if writing to the stream fails.
    pub async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()> {
        self.0.write_vectored(pieces).await
    }

    /// # Errors
    ///
    /// Returns an error if pumping data fails.
    pub async fn try_pump_from(
        &mut self,
        input: &mut RustAsyncInputStream,
        amount: usize,
    ) -> Result<usize> {
        match self.0.try_pump_from(input.0.as_mut(), amount).await {
            Some(result) => result,
            None => Ok(0), // Return 0 to indicate no optimization available
        }
    }

    /// # Errors
    ///
    /// Returns an error if checking disconnection status fails.
    pub async fn when_write_disconnected(&mut self) -> Result<()> {
        self.0.when_write_disconnected().await
    }
}

/// Opaque Rust type that can hold any `AsyncIoStream` implementation
pub struct RustAsyncIoStream(Box<dyn AsyncIoStream>);

impl RustAsyncIoStream {
    pub fn new<T: AsyncIoStream + 'static>(stream: T) -> Self {
        Self(Box::new(stream))
    }

    // FFI method implementations - AsyncInputStream part
    /// # Errors
    ///
    /// Returns an error if reading from the stream fails.
    pub async fn try_read(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize> {
        self.0.try_read(buffer, min_bytes).await
    }

    #[must_use]
    pub fn try_get_length(&self) -> usize {
        self.0.try_get_length().unwrap_or(0)
    }

    /// # Errors
    ///
    /// Returns an error if pumping data fails.
    pub async fn pump_to(
        &mut self,
        output: &mut RustAsyncOutputStream,
        amount: usize,
    ) -> Result<usize> {
        self.0.pump_to(output.0.as_mut(), amount).await
    }

    // FFI method implementations - AsyncOutputStream part
    /// # Errors
    ///
    /// Returns an error if writing to the stream fails.
    pub async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        self.0.write(buffer).await
    }

    /// # Errors
    ///
    /// Returns an error if writing to the stream fails.
    pub async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()> {
        self.0.write_vectored(pieces).await
    }

    /// # Errors
    ///
    /// Returns an error if pumping data fails.
    pub async fn try_pump_from(
        &mut self,
        input: &mut RustAsyncInputStream,
        amount: usize,
    ) -> Result<usize> {
        match self.0.try_pump_from(input.0.as_mut(), amount).await {
            Some(result) => result,
            None => Ok(0), // Return 0 to indicate no optimization available
        }
    }

    /// # Errors
    ///
    /// Returns an error if checking disconnection status fails.
    pub async fn when_write_disconnected(&mut self) -> Result<()> {
        self.0.when_write_disconnected().await
    }

    // FFI method implementations - AsyncIoStream specific
    /// # Errors
    ///
    /// Returns an error if shutting down the write end fails.
    pub async fn shutdown_write(&mut self) -> Result<()> {
        self.0.shutdown_write().await
    }

    pub fn abort_read(&mut self) {
        self.0.abort_read();
    }
}
