#include "own.h"

namespace kj_rs {
    void destroy_own(OwnVoid *own) {
        own->~Own();
    }
}  // namespace kj_rs
