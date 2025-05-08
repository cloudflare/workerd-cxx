#include <kj/async.h>
#include <kj-rs/promise.h>

namespace kj_rs {

kj::Promise<void> c_async_fn() {
    return kj::READY_NOW;
}

}