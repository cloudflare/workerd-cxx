#include "tests/nonnull_tests.h"
#include "tests/nonnull.rs.h"
#include <stdexcept>

namespace tests {

// Test function that receives NonNull<Resource> as *mut Resource
void take_resource_nonnull(Resource* ptr) {
    // Verify we received a non-null pointer
    if (ptr == nullptr) {
        throw std::runtime_error("Received null pointer");
    }
    // Verify the value
    if (ptr->value != 42) {
        throw std::runtime_error("Unexpected value");
    }
}

// Test function that returns a raw pointer (will become NonNull on Rust side)
Resource* create_resource() {
    static Resource resource{100};
    return &resource;
}

// Test function that reads value through NonNull pointer
std::int32_t get_resource_value(Resource* ptr) {
    if (ptr == nullptr) {
        throw std::runtime_error("Received null pointer");
    }
    return ptr->value;
}

} // namespace tests
