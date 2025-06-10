#pragma once

#include <cstdint>
#include <kj/debug.h>

namespace kj_rs {
	
class CppType {
public:
	CppType(uint64_t);
	~CppType();
	uint64_t cpptype_get() const;
	void cpptype_set(uint64_t val);
private:
	uint64_t data;
};

}
