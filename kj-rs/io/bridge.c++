#include "kj-rs/io/bridge.h"

#include "include/cxx.h"
#include "kj-rs/convert.h"
#include "kj-rs/io/ffi.rs.h"

#include <algorithm>

using namespace kj_rs;

namespace kj_rs_io {

namespace ffi {

CxxAsyncInputStream::CxxAsyncInputStream(kj::Own<kj::AsyncInputStream> stream)
    : stream(kj::mv(stream)) {}

kj::Promise<::std::size_t> CxxAsyncInputStream::try_read(
    ::rust::Slice<::std::uint8_t> buffer, ::std::size_t min_bytes) {
  return stream->tryRead(buffer.data(), min_bytes, buffer.size());
}

kj::Maybe<rust::usize> CxxAsyncInputStream::try_get_length() {
  return stream->tryGetLength();
}

kj::Promise<::std::uint64_t> CxxAsyncInputStream::pump_to(
    CxxAsyncOutputStream& output, ::std::uint64_t amount) {
  return stream->pumpTo(*output.stream, amount);
}

CxxAsyncOutputStream::CxxAsyncOutputStream(kj::Own<kj::AsyncOutputStream> stream)
    : stream(kj::mv(stream)) {}

kj::Promise<void> CxxAsyncOutputStream::write(::rust::Slice<const ::std::uint8_t> buffer) {
  return stream->write(from<Rust>(buffer));
}

kj::Promise<void> CxxAsyncOutputStream::write_vectored(
    ::rust::Slice<const ::rust::Slice<const ::std::uint8_t>> pieces) {
  // Convert rust slice of slices to kj::Array of ArrayPtrs
  // TODO: no alloc
  auto kj_pieces = kj::heapArray<kj::ArrayPtr<const kj::byte>>(pieces.size());

  for (size_t i = 0; i < pieces.size(); ++i) {
    kj_pieces[i] = from<Rust>(pieces[i]);
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
  return stream->tryRead(buffer.data(), min_bytes, buffer.size());
}

kj::Maybe<::std::size_t> CxxAsyncIoStream::try_get_length() {
  return stream->tryGetLength();
}

kj::Promise<::std::uint64_t> CxxAsyncIoStream::pump_to(
    CxxAsyncOutputStream& output, ::std::uint64_t amount) {
  return stream->pumpTo(*output.stream, amount);
}

// Methods inherited from AsyncOutputStream
kj::Promise<void> CxxAsyncIoStream::write(::rust::Slice<const ::std::uint8_t> buffer) {
  return stream->write(from<Rust>(buffer));
}

kj::Promise<void> CxxAsyncIoStream::write_vectored(
    ::rust::Slice<const ::rust::Slice<const ::std::uint8_t>> pieces) {
  auto kj_pieces = kj::heapArray<kj::ArrayPtr<const kj::byte>>(pieces.size());

  for (size_t i = 0; i < pieces.size(); ++i) {
    kj_pieces[i] = from<Rust>(pieces[i]);
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

}  // namespace ffi

}  // namespace kj_rs_io