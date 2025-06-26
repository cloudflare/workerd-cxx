#pragma once

#include "kj/refcount.h"
#include "test-own.h"
#include <cstdint>
#include <kj/debug.h>
#include <rust/cxx.h>

namespace kj_rs_demo {

class OpaqueRefcountedCxxClass: kj::Refcounted {
public:
  OpaqueCxxClass(uint64_t d) : data(d) {}
  ~OpaqueCxxClass() {}
  uint64_t getData() const { return this->data; }
  void setData(uint64_t val) { this->data = val; }

private:
  uint64_t data;
};

kj::Rc<OpaqueRefcountedCxxClass> cxx_kj_rc();
} // namespace kj_rs_demo
