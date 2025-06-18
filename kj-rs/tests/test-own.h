#pragma once

#include "kj/memory.h"
#include <cstdint>
#include <kj/debug.h>

namespace kj_rs_demo {

class OpaqueCxxClass {
public:
  OpaqueCxxClass(uint64_t data) : data(data) {}
  ~OpaqueCxxClass() {}
  uint64_t getData() const { return this->data; }
  void setData(uint64_t val) { this->data = val; }

private:
  uint64_t data;
};

// Forward declaration for Rust function, including the lib.rs.h caused problems
kj::Own<OpaqueCxxClass> modify_own_return(kj::Own<OpaqueCxxClass> cpp_own);

// Function declarations
kj::Own<OpaqueCxxClass> cxx_kj_own();
kj::Own<OpaqueCxxClass> null_kj_own();
void give_own_back(kj::Own<OpaqueCxxClass> own);
void modify_own_return_test();
kj::Own<OpaqueCxxClass> breaking_things();

} // namespace kj_rs_demo
