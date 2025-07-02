#include "test-maybe.h"
#include "kj/common.h"
#include "kj/debug.h"
#include <cstdint>

namespace kj_rs_demo {

kj::Maybe<int64_t> return_maybe() {
    kj::Maybe<int64_t> ret = kj::some(14);
    return kj::mv(ret);
}

}  // namespace kj_rs_demo
