#pragma once

#include <cstdint>

namespace kj_rs {
	
class CppType {
public:
	CppType(uint64_t);
	uint64_t cpptype_get() const;
private:
	uint64_t data;
};

}
