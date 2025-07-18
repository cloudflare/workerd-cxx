#include "test-maybe.h"

#include "kj-rs-demo/lib.rs.h"
#include "kj-rs/tests/lib.rs.h"
#include "kj/common.h"
#include "kj/debug.h"
#include "kj/memory.h"

#include <cstdint>

namespace kj_rs_demo {

kj::Maybe<Shared> return_maybe_shared_some() {
  return kj::Maybe<Shared>(Shared{14});
}

kj::Maybe<Shared> return_maybe_shared_none() {
  return kj::Maybe<Shared>(kj::none);
}

kj::Maybe<int64_t> return_maybe() {
  kj::Maybe<int64_t> ret = kj::some(14);
  return kj::mv(ret);
}

kj::Maybe<int64_t> return_maybe_none() {
  kj::Maybe<int64_t> ret = kj::none;
  return kj::mv(ret);
}

// Static var to return non-dangling pointer without heap allocating
int64_t var = 14;

static_assert(sizeof(kj::Maybe<const int64_t&>) == sizeof(const int64_t&));
kj::Maybe<const int64_t&> return_maybe_ref_none() {
  kj::Maybe<const int64_t&> ret = kj::none;
  return kj::mv(ret);
}

kj::Maybe<const int64_t&> return_maybe_ref_some() {
  const int64_t& val = var;
  kj::Maybe<const int64_t&> ret = kj::some(val);
  return kj::mv(ret);
}

kj::Maybe<kj::Own<OpaqueCxxClass>> return_maybe_own_none() {
  kj::Maybe<kj::Own<OpaqueCxxClass>> maybe = kj::none;
  return kj::mv(maybe);
}

kj::Maybe<kj::Own<OpaqueCxxClass>> return_maybe_own_some() {
  kj::Maybe<kj::Own<OpaqueCxxClass>> ret = kj::heap<OpaqueCxxClass>(14);
  return kj::mv(ret);
}

void test_maybe_reference_shared_own_driver() {
  kj::Maybe<kj::Own<OpaqueCxxClass>> maybe_own_some = return_maybe_own_some();
  uint64_t num = 15;
  kj::Maybe<uint64_t&> maybe_ref_some = kj::Maybe<uint64_t&>(num);
  kj::Maybe<Shared> maybe_shared_some = return_maybe_shared_some();

  auto maybe_own = take_maybe_own_ret(kj::mv(maybe_own_some));
  KJ_IF_SOME(i, maybe_own) {
    KJ_ASSERT(i->getData() == 42);
  } else {
    KJ_FAIL_ASSERT("Not reached");
  }
  take_maybe_own(kj::mv(maybe_own));

  auto maybe_ref = take_maybe_ref_ret(kj::mv(maybe_ref_some));
  take_maybe_ref(kj::mv(maybe_ref));

  auto maybe_shared = take_maybe_shared_ret(kj::mv(maybe_shared_some));
  KJ_IF_SOME(_, maybe_shared) {
    KJ_FAIL_ASSERT("Returns none, so unreached");
  }
  take_maybe_shared(kj::mv(maybe_shared));
}

}  // namespace kj_rs_demo
