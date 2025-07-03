#include "test-maybe.h"
#include "kj/common.h"
#include "kj/debug.h"
#include <cstdint>

namespace kj_rs_demo {

kj::Maybe<int64_t> return_maybe() {
    kj::Maybe<int64_t> ret = kj::some(14);
    return kj::mv(ret);
}

kj::Maybe<int64_t> return_maybe_none() {
    kj::Maybe<int64_t> ret = kj::none;
    return kj::mv(ret);
}

kj::Maybe<int64_t const*> return_maybe_ref() {
    return kj::none;
}

}  // namespace kj_rs_demo
