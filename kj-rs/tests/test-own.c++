#include "test-own.h"

namespace kj_rs_demo {

kj::Own<OpaqueCxxClass> cxx_kj_own() { 
  return kj::heap<OpaqueCxxClass>(42); 
}

kj::Own<OpaqueCxxClass> null_kj_own() { 
  return kj::Own<OpaqueCxxClass>(); 
}

void give_own_back(kj::Own<OpaqueCxxClass> own) {
  own->setData(37);
  KJ_ASSERT(own->getData() == 37);
}

void modify_own_return_test() {
  auto owned = kj::heap<OpaqueCxxClass>(17);
  auto returned = modify_own_return(kj::mv(owned));
  KJ_ASSERT(returned->getData() == 72);
}

kj::Own<OpaqueCxxClass> breaking_things() {
  auto own0 = kj::heap<OpaqueCxxClass>(42);
  auto own1 = kj::heap<OpaqueCxxClass>(72);
  auto own2 = own0.attach(kj::mv(own1));
  return own2;
}

} // namespace kj_rs_demo
