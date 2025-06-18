#pragma once

#include <kj/memory.h>

namespace kj_rs {
    using OwnVoid = kj::Own<void>;

    void destroy_own(OwnVoid *own);
}  // namespace kj_rs
