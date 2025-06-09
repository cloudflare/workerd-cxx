#include "kj-rs/io/tests.h"

#include "kj-rs/convert.h"
#include "kj-rs/io/bridge.h"
#include "kj-rs/io/ffi.rs.h"
#include "kj-rs/io/tests.rs.h"

#include <kj/async-io.h>
#include <kj/debug.h>
#include <kj/test.h>

#include <cstring>
#include <memory>

using namespace kj_rs;

namespace kj_rs_io_test {

template <typename T>
class RustAsyncInputStream: public kj::AsyncInputStream {
 public:
  explicit RustAsyncInputStream(T&& impl): impl(kj::mv(impl)) {}
  virtual ~RustAsyncInputStream() = default;

  // kj::AsyncInputStream interface
  kj::Promise<size_t> tryRead(void* buffer, size_t minBytes, size_t maxBytes) override {
    return impl->try_read(
        kj::arrayPtr(reinterpret_cast<uint8_t*>(buffer), maxBytes).as<RustMutable>(), minBytes);
  }

  kj::Maybe<uint64_t> tryGetLength() override {
    return impl->try_get_length();
  }

  // kj::Promise<uint64_t> pumpTo(kj::AsyncOutputStream& output, uint64_t amount) override {
  //   return impl->pumpTo(output, amount);
  // }

 private:
  T impl;
};

template <typename T>
class RustAsyncOutputStream: public kj::AsyncOutputStream {
 public:
  explicit RustAsyncOutputStream(T&& impl): impl(kj::mv(impl)) {}
  virtual ~RustAsyncOutputStream() = default;

  // kj::AsyncOutputStream interface
  kj::Promise<void> write(kj::ArrayPtr<const kj::byte> buffer) override {
    return impl->write(buffer);
  }

  kj::Promise<void> write(kj::ArrayPtr<const kj::ArrayPtr<const kj::byte>> pieces) override {
    return impl->write(pieces);
  }

  kj::Maybe<kj::Promise<uint64_t>> tryPumpFrom(
      kj::AsyncInputStream& input, uint64_t amount) override {
    return impl->tryPumpFrom(input, amount);
  }

  kj::Promise<void> whenWriteDisconnected() override {
    return impl->whenWriteDisconnected();
  }

 private:
  T impl;
};

template <typename T>
class RustAsyncIoStream: public kj::AsyncIoStream {
 public:
  explicit RustAsyncIoStream(T&& impl): impl(kj::mv(impl)) {}
  virtual ~RustAsyncIoStream() = default;

  // kj::AsyncInputStream interface
  kj::Promise<size_t> tryRead(void* buffer, size_t minBytes, size_t maxBytes) override {
    return impl->tryRead(buffer, minBytes, maxBytes);
  }

  kj::Maybe<uint64_t> tryGetLength() override {
    return impl->tryGetLength();
  }

  kj::Promise<uint64_t> pumpTo(kj::AsyncOutputStream& output, uint64_t amount) override {
    return impl->pumpTo(output, amount);
  }

  // kj::AsyncOutputStream interface
  kj::Promise<void> write(kj::ArrayPtr<const kj::byte> buffer) override {
    return impl->write(buffer);
  }

  kj::Promise<void> write(kj::ArrayPtr<const kj::ArrayPtr<const kj::byte>> pieces) override {
    return impl->write(pieces);
  }

  kj::Maybe<kj::Promise<uint64_t>> tryPumpFrom(
      kj::AsyncInputStream& input, uint64_t amount) override {
    return impl->tryPumpFrom(input, amount);
  }

  kj::Promise<void> whenWriteDisconnected() override {
    return impl->whenWriteDisconnected();
  }

  // kj::AsyncIoStream interface
  void shutdownWrite() override {
    impl->shutdownWrite();
  }

  void abortRead() override {
    impl->abortRead();
  }

 private:
  T impl;
};

// Simple input stream that reads from array data
class ArrayInputStream: public kj::AsyncInputStream {
 public:
  ArrayInputStream(kj::ArrayPtr<const kj::byte> data): data(data) {}
  virtual ~ArrayInputStream() = default;

  kj::Promise<size_t> tryRead(void* buffer, size_t minBytes, size_t maxBytes) override {
    size_t toRead = std::min(maxBytes, data.size());

    if (toRead == 0) {
      return size_t(0);  // EOF
    }

    memcpy(buffer, data.begin(), toRead);
    data = data.slice(toRead);
    return toRead;
  }

  kj::Maybe<uint64_t> tryGetLength() override {
    return data.size();
  }

 private:
  kj::ArrayPtr<const kj::byte> data;
};

// Simple output stream that writes to a kj::Vector
class VectorOutputStream: public kj::AsyncOutputStream {
 public:
  VectorOutputStream() = default;
  virtual ~VectorOutputStream() = default;

  kj::Promise<void> write(kj::ArrayPtr<const kj::byte> buffer) override {
    data.addAll(buffer);
    return kj::READY_NOW;
  }

  kj::Promise<void> write(kj::ArrayPtr<const kj::ArrayPtr<const kj::byte>> pieces) override {
    for (auto piece: pieces) {
      data.addAll(piece);
    }
    return kj::READY_NOW;
  }

  kj::Promise<void> whenWriteDisconnected() override {
    return kj::NEVER_DONE;  // Never disconnected
  }

  const kj::Vector<kj::byte>& getData() const {
    return data;
  }

  void clear() {
    data.clear();
  }

 private:
  kj::Vector<kj::byte> data;
};

// C++ implementation of FNV-1a hash algorithm (matching Rust implementation)
kj::Promise<uint64_t> computeStreamHash(kj::AsyncInputStream& stream) {
  static constexpr uint64_t FNV_OFFSET_BASIS = 14695981039346656037ULL;
  static constexpr uint64_t FNV_PRIME = 1099511628211ULL;

  uint64_t hash = FNV_OFFSET_BASIS;
  auto buffer = kj::heapArray<kj::byte>(4096);

  for (;;) {
    size_t bytesRead = co_await stream.tryRead(buffer.begin(), 1, buffer.size());

    if (bytesRead == 0) {
      co_return hash;  // EOF
    }

    // FNV-1a hash algorithm
    for (size_t i = 0; i < bytesRead; i++) {
      hash ^= static_cast<uint64_t>(buffer[i]);
      hash *= FNV_PRIME;
    }
  }
}

KJ_TEST("Read C++ ArrayInputStream in C++") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto testData = "Hello, World!"_kjb;

  auto stream = ArrayInputStream(testData);
  auto hash = computeStreamHash(stream).wait(waitScope);
  KJ_EXPECT(hash == 7993990320990026836);

  auto stream2 = ArrayInputStream(testData);
  auto hash2 = computeStreamHash(stream2).wait(waitScope);
  KJ_EXPECT(hash == hash2);
}

KJ_TEST("Read C++ ArrayInputStream in Rust") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto testData = "Hello, World!"_kjb;

  auto stream = kj_rs_io::ffi::CxxAsyncInputStream(kj::heap<ArrayInputStream>(testData));
  auto hash = compute_stream_hash_ffi(stream).wait(waitScope);

  KJ_EXPECT(hash == 7993990320990026836);

  auto stream2 = kj_rs_io::ffi::CxxAsyncInputStream(kj::heap<ArrayInputStream>(testData));
  auto hash2 = compute_stream_hash_ffi(stream2).wait(waitScope);

  KJ_EXPECT(hash == hash2);
}

KJ_TEST("Write to C++ OutputStream from Rust") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto vectorStream = kj::heap<VectorOutputStream>();
  auto* streamPtr = vectorStream.get();

  auto stream = kj_rs_io::ffi::CxxAsyncOutputStream(kj::mv(vectorStream));

  // Generate 100 bytes of pseudorandom data
  generate_prng_ffi(stream, 100).wait(waitScope);

  // Check that data was written
  const auto& data = streamPtr->getData();
  KJ_EXPECT(data.size() == 100);

  // Test that the data is deterministic by comparing with another stream
  auto vectorStream2 = kj::heap<VectorOutputStream>();
  auto* streamPtr2 = vectorStream2.get();
  auto stream2 = kj_rs_io::ffi::CxxAsyncOutputStream(kj::mv(vectorStream2));

  generate_prng_ffi(stream2, 100).wait(waitScope);

  const auto& data2 = streamPtr2->getData();
  KJ_EXPECT(data.size() == data2.size());

  // Compare the data byte by byte
  for (size_t i = 0; i < data.size(); i++) {
    KJ_EXPECT(data[i] == data2[i], "Data should be deterministic", i, data[i], data2[i]);
  }
}

KJ_TEST("Write large data to C++ OutputStream from Rust") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto vectorStream = kj::heap<VectorOutputStream>();
  auto* streamPtr = vectorStream.get();

  auto stream = kj_rs_io::ffi::CxxAsyncOutputStream(kj::mv(vectorStream));

  // Generate 2048 * 2048 bytes (multiple chunks)
  generate_prng_ffi(stream, 2048 * 2048).wait(waitScope);

  // Check that data was written
  const auto& data = streamPtr->getData();
  KJ_EXPECT(data.size() == 2048 * 2048);

  // Basic check that the data varies (not all zeros)
  size_t zeroCount = 0;
  for (auto byte: data) {
    if (byte == 0) zeroCount++;
  }
  KJ_EXPECT(zeroCount < data.size() / 10, "Data should vary, less than 10% zeros");
}

KJ_TEST("Test Rust InputStream length - known size") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto testData = "Hello, World!"_kjb;
  auto stream = RustAsyncInputStream(mock_input_stream(testData.as<Rust>()));

  KJ_EXPECT(KJ_ASSERT_NONNULL(stream.tryGetLength()) == testData.size());
}

KJ_TEST("Test Rust InputStream length - unknown size") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto testData = "Hello, World!"_kjb;
  auto stream = RustAsyncInputStream(mock_input_stream_no_size(testData.as<Rust>()));

  // For unknown size, tryGetLength should return none
  KJ_EXPECT(stream.tryGetLength() == kj::none);
}

KJ_TEST("Read Rust InputStream from C++") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto testData = "Hello, World!"_kjb;

  auto stream = RustAsyncInputStream(mock_input_stream(testData.as<Rust>()));
  auto hash = computeStreamHash(stream).wait(waitScope);
  KJ_EXPECT(hash == 7993990320990026836);
}

KJ_TEST("Test Rust InputStream with error") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  // Create a stream that returns read errors
  auto stream = RustAsyncInputStream(mock_input_stream_with_error());

  // Reading from the stream should throw an exception
  uint8_t buffer[10];
  KJ_EXPECT_THROW_MESSAGE("read error", stream.tryRead(buffer, 1, 10).wait(waitScope));
}

}  // namespace kj_rs_io_test