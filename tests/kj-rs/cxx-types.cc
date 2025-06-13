#include "cxx-types.h"

namespace kj_rs {

kj::Own<CxxType> cxx_kj_own() { 
  return kj::heap<CxxType>(42); 
}

kj::Own<CxxType> null_kj_own() { 
  return kj::Own<CxxType>(); 
}

void give_own_back(kj::Own<CxxType> own) {
  own->setData(37);
  KJ_ASSERT(own->getData() == 37);
}

void modify_own_return_test() {
  auto owned = kj::heap<CxxType>(17);
  auto returned = modify_own_return(kj::mv(owned));
  KJ_ASSERT(returned->getData() == 72);
}

kj::Own<CxxType> breaking_things() {
  auto own0 = kj::heap<CxxType>(42);
  auto own1 = kj::heap<CxxType>(72);
  auto own2 = own0.attach(kj::mv(own1));
  return own2;
}

} // namespace kj_rs