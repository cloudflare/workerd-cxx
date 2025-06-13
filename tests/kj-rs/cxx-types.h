#pragma once

#include "kj/memory.h"
#include <cstdint>
#include <kj/debug.h>

namespace kj_rs {

class CxxType {
public:
  CxxType(uint64_t data) : data(data) {}
  ~CxxType() {}
  uint64_t getData() const { return this->data; }
  void setData(uint64_t val) { this->data = val; }

private:
  uint64_t data;
};

// Forward declaration for Rust function
kj::Own<CxxType> modify_own_return(kj::Own<CxxType> cpp_own);

// Function declarations
kj::Own<CxxType> cxx_kj_own();
kj::Own<CxxType> null_kj_own();
void give_own_back(kj::Own<CxxType> own);
void modify_own_return_test();
kj::Own<CxxType> breaking_things();

} // namespace kj_rs
