use std::pin::Pin;

use async_trait::async_trait;
use kj_rs::KjMaybe;

use crate::{AsyncInputStream, AsyncIoStream, AsyncOutputStream};

type Result<T> = std::io::Result<T>;

#[cxx::bridge(namespace = "kj_rs_io::ffi")]
#[allow(clippy::needless_lifetimes)]
pub mod bridge {
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

        fn try_get_length(self: &RustAsyncInputStream) -> KjMaybe<usize>;

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

        fn try_get_length(self: &RustAsyncIoStream) -> KjMaybe<usize>;

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
        fn try_get_length(self: Pin<&mut CxxAsyncInputStream>) -> KjMaybe<usize>;

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
        fn try_get_length(self: Pin<&mut CxxAsyncIoStream>) -> KjMaybe<usize>;

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
    pub fn try_get_length(&self) -> KjMaybe<usize> {
        self.0.try_get_length().into()
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

// unsafe impl cxx::ExternType for RustAsyncInputStream {
//     type Id = type_id!("kj_rs_io::ffi::RustAsyncInputStream");
//     type Kind = cxx::kind::Opaque;
// }

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
    pub fn try_get_length(&self) -> KjMaybe<usize> {
        self.0.try_get_length().into()
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

/// Helper function to convert `cxx::Exception` to `std::io::Error`
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn cxx_to_io_error(e: cxx::KjException) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Exception: {}", e.what()),
    )
}

/// Rust wrapper for the `CxxAsyncInputStream` FFI type
pub struct CxxAsyncInputStream<'a>(Pin<&'a mut bridge::CxxAsyncInputStream>);

impl<'a> CxxAsyncInputStream<'a> {
    #[must_use]
    pub fn new(inner: Pin<&'a mut bridge::CxxAsyncInputStream>) -> Self {
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

    async fn pump_to(
        &mut self,
        output: &mut dyn AsyncOutputStream,
        amount: usize,
    ) -> Result<usize> {
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
pub struct CxxAsyncOutputStream<'a>(Pin<&'a mut bridge::CxxAsyncOutputStream>);

impl<'a> CxxAsyncOutputStream<'a> {
    #[must_use]
    pub fn new(inner: Pin<&'a mut bridge::CxxAsyncOutputStream>) -> Self {
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
pub struct CxxAsyncIoStream<'a>(Pin<&'a mut bridge::CxxAsyncIoStream>);

impl<'a> CxxAsyncIoStream<'a> {
    #[must_use]
    pub fn new(inner: Pin<&'a mut bridge::CxxAsyncIoStream>) -> Self {
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

    async fn pump_to(
        &mut self,
        output: &mut dyn AsyncOutputStream,
        amount: usize,
    ) -> Result<usize> {
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
