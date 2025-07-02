#include "kj-rs/io/bridge.h"

#include "kj-rs/io/lib.rs.h"

#include <algorithm>

namespace kj_rs_io {

CxxAsyncInputStream::CxxAsyncInputStream(kj::Own<kj::AsyncInputStream> stream)
    : stream(kj::mv(stream)) {}

kj::Promise<::std::size_t> CxxAsyncInputStream::try_read(
    ::rust::Slice<::std::uint8_t> buffer, ::std::size_t min_bytes) {
  // Convert rust::Slice to void* and size_t for KJ interface
  void* buf_ptr = const_cast<::std::uint8_t*>(buffer.data());
  ::std::size_t buf_size = buffer.size();

  // Use buffer size as max_bytes
  ::std::size_t actual_min = std::min(min_bytes, buf_size);

  return stream->tryRead(buf_ptr, actual_min, buf_size);
}

::std::uint64_t CxxAsyncInputStream::try_get_length() {
  auto maybe_length = stream->tryGetLength();
  KJ_IF_SOME(length, maybe_length) {
    return length;
  } else {
    return 0;  // Return 0 if unknown
  }
}

kj::Promise<::std::uint64_t> CxxAsyncInputStream::pump_to(
    CxxAsyncOutputStream& output, ::std::uint64_t amount) {
  return stream->pumpTo(*output.stream, amount);
}

CxxAsyncOutputStream::CxxAsyncOutputStream(kj::Own<kj::AsyncOutputStream> stream)
    : stream(kj::mv(stream)) {}

kj::Promise<void> CxxAsyncOutputStream::write(::rust::Slice<const ::std::uint8_t> buffer) {
  // Convert rust::Slice to kj::ArrayPtr
  auto array_ptr =
      kj::ArrayPtr<const kj::byte>(reinterpret_cast<const kj::byte*>(buffer.data()), buffer.size());
  return stream->write(array_ptr);
}

kj::Promise<void> CxxAsyncOutputStream::write_vectored(
    ::rust::Slice<const ::rust::Slice<const ::std::uint8_t>> pieces) {
  // Convert rust slice of slices to kj::Array of ArrayPtrs
  auto kj_pieces = kj::heapArray<kj::ArrayPtr<const kj::byte>>(pieces.size());

  for (size_t i = 0; i < pieces.size(); ++i) {
    const auto& piece = pieces[i];
    kj_pieces[i] =
        kj::ArrayPtr<const kj::byte>(reinterpret_cast<const kj::byte*>(piece.data()), piece.size());
  }

  return stream->write(kj::ArrayPtr<const kj::ArrayPtr<const kj::byte>>(kj_pieces));
}

kj::Promise<::std::uint64_t> CxxAsyncOutputStream::try_pump_from(
    CxxAsyncInputStream& input, ::std::uint64_t amount) {
  auto maybe_pump_promise = stream->tryPumpFrom(*input.stream, amount);
  KJ_IF_SOME(pump_promise, maybe_pump_promise) {
    return kj::mv(pump_promise);
  } else {
    // Return a resolved promise with 0 to indicate not supported
    return ::std::uint64_t(0);
  }
}

kj::Promise<void> CxxAsyncOutputStream::when_write_disconnected() {
  return stream->whenWriteDisconnected();
}

CxxAsyncIoStream::CxxAsyncIoStream(kj::Own<kj::AsyncIoStream> stream): stream(kj::mv(stream)) {}

// Methods inherited from AsyncInputStream
kj::Promise<::std::size_t> CxxAsyncIoStream::try_read(
    ::rust::Slice<::std::uint8_t> buffer, ::std::size_t min_bytes) {
  void* buf_ptr = const_cast<::std::uint8_t*>(buffer.data());
  ::std::size_t buf_size = buffer.size();

  ::std::size_t actual_min = std::min(min_bytes, buf_size);

  return stream->tryRead(buf_ptr, actual_min, buf_size);
}

::std::uint64_t CxxAsyncIoStream::try_get_length() {
  auto maybe_length = stream->tryGetLength();
  KJ_IF_SOME(length, maybe_length) {
    return length;
  } else {
    return 0;
  }
}

kj::Promise<::std::uint64_t> CxxAsyncIoStream::pump_to(
    CxxAsyncOutputStream& output, ::std::uint64_t amount) {
  return stream->pumpTo(*output.stream, amount);
}

// Methods inherited from AsyncOutputStream
kj::Promise<void> CxxAsyncIoStream::write(::rust::Slice<const ::std::uint8_t> buffer) {
  auto array_ptr =
      kj::ArrayPtr<const kj::byte>(reinterpret_cast<const kj::byte*>(buffer.data()), buffer.size());
  return stream->write(array_ptr);
}

kj::Promise<void> CxxAsyncIoStream::write_vectored(
    ::rust::Slice<const ::rust::Slice<const ::std::uint8_t>> pieces) {
  auto kj_pieces = kj::heapArray<kj::ArrayPtr<const kj::byte>>(pieces.size());

  for (size_t i = 0; i < pieces.size(); ++i) {
    const auto& piece = pieces[i];
    kj_pieces[i] =
        kj::ArrayPtr<const kj::byte>(reinterpret_cast<const kj::byte*>(piece.data()), piece.size());
  }

  return stream->write(kj::ArrayPtr<const kj::ArrayPtr<const kj::byte>>(kj_pieces));
}

kj::Promise<::std::uint64_t> CxxAsyncIoStream::try_pump_from(
    CxxAsyncInputStream& input, ::std::uint64_t amount) {
  auto maybe_pump_promise = stream->tryPumpFrom(*input.stream, amount);
  KJ_IF_SOME(pump_promise, maybe_pump_promise) {
    return kj::mv(pump_promise);
  } else {
    return ::std::uint64_t(0);
  }
}

kj::Promise<void> CxxAsyncIoStream::when_write_disconnected() {
  return stream->whenWriteDisconnected();
}

// Methods specific to AsyncIoStream
void CxxAsyncIoStream::shutdown_write() {
  stream->shutdownWrite();
}

void CxxAsyncIoStream::abort_read() {
  stream->abortRead();
}

}  // namespace kj_rs_io