#pragma once

#include "kj/common.h"
#include <cstdint>

namespace kj_rs_demo {

kj::Maybe<int64_t> return_maybe();
kj::Maybe<int64_t> return_maybe_none();

kj::Maybe<int64_t const*> return_maybe_ref();

}  // namespace kj_rs_demo
