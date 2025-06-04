#pragma once

#include <cstdint>
#include <kj/async.h>
#include "cxx-types.h"
#include "tests/kj-rs/lib.rs.h"

namespace kj_rs {

kj::Promise<void> c_async_void_fn();
kj::Promise<int64_t> c_async_int_fn();
kj::Promise<Shared> c_async_struct_fn();
kj::Own<CppType> cpp_kj_own();

} // namespace kj_rs
