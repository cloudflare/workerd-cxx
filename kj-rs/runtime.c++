#include "kj-rs/runtime.h"

#include <kj/debug.h>

namespace kj_rs {

KjRuntimeImpl::KjRuntimeImpl() {}

KjRuntimeImpl::~KjRuntimeImpl() {
  // Ensure WaitScope is destroyed before EventLoop.
  waitScope = kj::none;
}

void KjRuntimeImpl::enterScope() {
  KJ_REQUIRE(waitScope == kj::none, "enterScope() called while already in scope");
  waitScope.emplace(loop);
}

void KjRuntimeImpl::leaveScope() {
  KJ_REQUIRE(waitScope != kj::none, "leaveScope() called while not in scope");
  waitScope = kj::none;
}

uint KjRuntimeImpl::poll() {
  KJ_IF_SOME(ws, waitScope) {
    return ws.poll();
  } else {
    KJ_FAIL_REQUIRE("poll() called without entering scope");
  }
}

bool KjRuntimeImpl::isRunnable() {
  return loop.isRunnable();
}

std::unique_ptr<KjRuntimeImpl> newKjRuntime() {
  return std::make_unique<KjRuntimeImpl>();
}

}  // namespace kj_rs
