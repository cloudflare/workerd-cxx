use std::{ffi::c_void, os::fd::RawFd};

use async_trait::async_trait;
use libc::{sockaddr, socklen_t};

use crate::Result;

/// Asynchronous equivalent of `InputStream`.
///
/// This trait corresponds to the C++ `kj::AsyncInputStream` class and provides
/// asynchronous reading capabilities with KJ promise integration.
#[async_trait(?Send)]
pub trait AsyncInputStream {
    /// Read at least `min_bytes` from the stream. Performs partial read if there is not enough data.
    /// Returns total number of bytes read. Return value less than `min_bytes` indicates EOF.
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
    /// The default implementation first tries calling output.tryPumpFrom(), but if that fails, it
    /// performs a naive pump by allocating a buffer and reading to it / writing from it in a loop.
    async fn pump_to(&mut self, output: &mut dyn AsyncOutputStream, amount: usize)
    -> Result<usize>;

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
    fn try_tee(&mut self, _limit: usize) -> Option<Box<dyn AsyncInputStream>> {
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
    async fn write_vectored(&mut self, pieces: &[&[u8]]) -> Result<()> {
        for piece in pieces {
            self.write(piece).await?;
        }
        Ok(())
    }

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
    unsafe fn getsockopt(
        &self,
        _level: i32,
        _option: i32,
        _optval: *mut c_void,
        _optlen: *mut socklen_t,
    ) -> Result<()> {
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
    unsafe fn getsockname(
        &self,
        _address: *mut sockaddr,
        _address_len: *mut socklen_t,
    ) -> Result<()> {
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
    fn getpeername(&self, _address: *mut sockaddr, _address_len: *mut socklen_t) -> Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "getpeername not supported",
        ))
    }

    /// Get the underlying Unix file descriptor, if any.
    ///
    /// Returns None if this object actually isn't wrapping a file descriptor.
    fn get_fd(&self) -> Option<RawFd> {
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

pub type AncillaryMessageHandler = Box<dyn FnMut(&[AncillaryMessage]) + 'static>;
