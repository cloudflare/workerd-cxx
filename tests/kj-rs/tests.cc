#include "tests.h"
#include <cassert>

namespace kj_rs {

CppType::CppType(uint64_t val) : data(val) {}

uint64_t CppType::cpptype_get() const {
	return this->data;
}

kj::Promise<void> c_async_void_fn() { return kj::READY_NOW; }

kj::Promise<int64_t> c_async_int_fn() { return 42; }

kj::Promise<Shared> c_async_struct_fn() { return Shared{42}; }

void call() {
    assert(16 == sizeof(kj::Own<int64_t>));
    assert(8 == sizeof(std::unique_ptr<int64_t>));
    assert(8 == sizeof(kj::Own<int64_t, void*(*)>));
    assert(16 == sizeof(std::unique_ptr<int64_t, void*(*)>));
}
kj::Own<CppType> cpp_kj_own() {
    return kj::heap<CppType>(42); 
}

} // namespace kj_rs
