#pragma once

#include <kj/memory.h>

namespace kj_rs {
    extern "C" {
        void cxxbridge$kjrs$own$drop(void* own);
    }
}  // namespace kj_rs
