#pragma once

namespace kj_rs_demo {
  struct Shared;
}

#include "kj-rs-demo/lib.rs.h"

#include "kj/common.h"

namespace kj_rs_demo {

kj::Maybe<int64_t> shared_access(Shared shared);

}  // namespace kj_rs_demo
