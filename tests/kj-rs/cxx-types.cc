#include "cxx-types.h"

namespace kj_rs {
	
CppType::CppType(uint64_t val) : data(val) {}

uint64_t CppType::cpptype_get() const {
	return this->data;
}

void CppType::cpptype_set(uint64_t val) {
	this->data = val;
}

}
