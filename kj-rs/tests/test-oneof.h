#pragma once

#include "kj-rs-demo/lib.rs.h"
#include "kj-rs/lib.rs.h"
#include "kj/one-of.h"

#include <cstdint>
#include <type_traits>

namespace kj_rs_demo {
static_assert(std::is_standard_layout<kj::OneOf<uint32_t, size_t>>::value, "");
static_assert(sizeof(kj::OneOf<uint32_t, size_t>) == 16, "");
static_assert(sizeof(kj::OneOf<uint32_t, uint32_t>) == 16, "");

kj::OneOf<uint32_t, size_t> new_oneof();
}  // namespace kj_rs_demo
