#pragma once

#include "kj-rs-demo/lib.rs.h"
#include "kj/common.h"

#include <cstdint>

namespace kj_rs_demo {

kj::Maybe<Shared> return_maybe_shared_some();
kj::Maybe<Shared> return_maybe_shared_none();

kj::Maybe<int64_t> return_maybe();
kj::Maybe<int64_t> return_maybe_none();

kj::Maybe<const int64_t&> return_maybe_ref_some();
kj::Maybe<const int64_t&> return_maybe_ref_none();

kj::Maybe<kj::Own<OpaqueCxxClass>> return_maybe_own_none();
kj::Maybe<kj::Own<OpaqueCxxClass>> return_maybe_own_some();

void test_maybe_reference_shared_own_driver();

}  // namespace kj_rs_demo
