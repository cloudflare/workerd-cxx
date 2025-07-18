#include "test-refcount.h"

namespace kj_rs_demo {
kj::Rc<OpaqueRefcountedClass> get_rc() {
	return kj::rc<OpaqueRefcountedClass>(15);
}

kj::Arc<OpaqueAtomicRefcountedClass> get_arc() {
	return kj::arc<OpaqueAtomicRefcountedClass>(16);
}
}
