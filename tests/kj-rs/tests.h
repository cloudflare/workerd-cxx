#pragma once

#include <cstdint>
#include <kj/async.h>
#include "tests/kj-rs/lib.rs.h"

namespace kj_rs {

class CppType {
public:
	CppType(uint64_t);
	uint64_t cpptype_get() const;
private:
	uint64_t data;
};

kj::Promise<void> c_async_void_fn();
kj::Promise<int64_t> c_async_int_fn();
kj::Promise<Shared> c_async_struct_fn();
kj::Own<CppType> cpp_kj_own();
void call();

} // namespace kj_rs
