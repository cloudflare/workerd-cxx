#include "test-maybe.h"
#include "kj/common.h"
#include "kj/debug.h"
#include <cstdint>

namespace kj_rs_demo {

kj::Maybe<int64_t> shared_access(Shared shared) {
    kj::Maybe<int64_t> ret = kj::some(14);
    KJ_ASSERT(false);
    return ret;
}

}  // namespace kj_rs_demo
