#include "test-oneof.h"

namespace kj_rs_demo {
kj::OneOf<uint32_t, size_t> new_oneof() {
  return uint32_t(12);
}
}  // namespace kj_rs_demo
