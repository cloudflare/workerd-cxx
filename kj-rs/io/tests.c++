#include "kj-rs/io/tests.h"

#include "kj-rs/io/tests.rs.h"

#include <kj/async-io.h>
#include <kj/debug.h>
#include <kj/test.h>

#include <cstring>
#include <memory>

namespace kj_rs_io_test {

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

// C++ implementation of FNV-1a hash algorithm (matching Rust implementation)
kj::Promise<uint64_t> computeStreamHash(kj::Own<kj::AsyncInputStream> stream) {
  static constexpr uint64_t FNV_OFFSET_BASIS = 14695981039346656037ULL;
  static constexpr uint64_t FNV_PRIME = 1099511628211ULL;
  
  uint64_t hash = FNV_OFFSET_BASIS;
  auto buffer = kj::heapArray<kj::byte>(4096);
  
  for (;;) {
    size_t bytesRead = co_await stream->tryRead(buffer.begin(), 1, buffer.size());
    
    if (bytesRead == 0) {
      co_return hash; // EOF
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

  auto hash = computeStreamHash(kj::heap<ArrayInputStream>(testData)).wait(waitScope);
  KJ_EXPECT(hash == 7993990320990026836);

  auto hash2 = computeStreamHash(kj::heap<ArrayInputStream>(testData)).wait(waitScope);
  KJ_EXPECT(hash == hash2);
}

KJ_TEST("Read C++ ArrayInputStream in Rust") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto testData = "Hello, World!"_kjb;

  auto stream = kj_rs_io::CxxAsyncInputStream(kj::heap<ArrayInputStream>(testData));
  auto hash = compute_stream_hash_ffi(stream).wait(waitScope);

  KJ_EXPECT(hash == 7993990320990026836);

  auto stream2 = kj_rs_io::CxxAsyncInputStream(kj::heap<ArrayInputStream>(testData));
  auto hash2 = compute_stream_hash_ffi(stream2).wait(waitScope);

  KJ_EXPECT(hash == hash2);
}

}  // namespace kj_rs_io_test