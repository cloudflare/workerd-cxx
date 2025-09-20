#pragma once

#include "kj-rs/convert.h"
#include "kj-rs/kj-rs.h"

#include <rust/cxx.h>

#include <kj/async-io.h>
#include <kj/memory.h>

#include <cstdint>

using namespace kj_rs;

namespace kj_rs_io {

// Forward declarations
namespace ffi {
struct CxxAsyncOutputStream;

// Wrapper structs for KJ async stream types to provide controlled FFI interface
struct CxxAsyncInputStream {
  kj::Own<kj::AsyncInputStream> stream;

  explicit CxxAsyncInputStream(kj::Own<kj::AsyncInputStream> stream);

  // Methods corresponding to KJ AsyncInputStream interface
  kj::Promise<::std::size_t> try_read(
      ::rust::Slice<::std::uint8_t> buffer, ::std::size_t min_bytes);

  kj::Maybe<::std::size_t> try_get_length();
  kj::Promise<::std::uint64_t> pump_to(CxxAsyncOutputStream& output, ::std::uint64_t amount);
};

struct CxxAsyncOutputStream {
  kj::Own<kj::AsyncOutputStream> stream;

  explicit CxxAsyncOutputStream(kj::Own<kj::AsyncOutputStream> stream);

  // Methods corresponding to KJ AsyncOutputStream interface
  kj::Promise<void> write(::rust::Slice<const ::std::uint8_t> buffer);
  kj::Promise<void> write_vectored(::rust::Slice<const ::rust::Slice<const ::std::uint8_t>> pieces);

  // todo: optional
  kj::Promise<::std::uint64_t> try_pump_from(CxxAsyncInputStream& input, ::std::uint64_t amount);
  kj::Promise<void> when_write_disconnected();
};

struct CxxAsyncIoStream {
  kj::Own<kj::AsyncIoStream> stream;

  explicit CxxAsyncIoStream(kj::Own<kj::AsyncIoStream> stream);

  // Methods inherited from AsyncInputStream
  kj::Promise<::std::size_t> try_read(
      ::rust::Slice<::std::uint8_t> buffer, ::std::size_t min_bytes);
  kj::Maybe<::std::size_t> try_get_length();
  kj::Promise<::std::uint64_t> pump_to(CxxAsyncOutputStream& output, ::std::uint64_t amount);

  // Methods inherited from AsyncOutputStream
  kj::Promise<void> write(::rust::Slice<const ::std::uint8_t> buffer);
  kj::Promise<void> write_vectored(::rust::Slice<const ::rust::Slice<const ::std::uint8_t>> pieces);
  // todo: optional
  kj::Promise<::std::uint64_t> try_pump_from(CxxAsyncInputStream& input, ::std::uint64_t amount);
  kj::Promise<void> when_write_disconnected();

  // Methods corresponding to KJ AsyncIoStream interface
  void shutdown_write();
  void abort_read();
};

}  // namespace ffi

}  // namespace kj_rs_io