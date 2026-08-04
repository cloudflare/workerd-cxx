#pragma once
#include <cstdint>

namespace tests {

struct Resource;

struct NativeResource {
    std::int32_t data;
};

// Test function declarations
void take_resource_nonnull(Resource* ptr);
Resource* create_resource();
std::int32_t get_resource_value(Resource* ptr);

} // namespace tests
