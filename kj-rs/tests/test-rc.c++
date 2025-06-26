#include "test-rc.h"
#include "test-own.h"

namespace kj_rs_demo {
    kj::Rc<OpaqueRefcountedCxxClass> cxx_kj_rc() {
         return kj::rc<OpaqueRefcountedCxxClass>(12);
	}	
}
