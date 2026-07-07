#pragma once

#include <kj/async.h>
#include <kj/common.h>
#include <kj/memory.h>

#include <memory>

namespace kj_rs {

// A KJ event loop that can be driven from Rust.
//
// Between enterScope() and leaveScope() calls, the EventLoop is bound to the calling thread
// via a kj::WaitScope. Outside this window, the EventLoop is unbound and can be transferred
// to a different thread.
//
// Typical usage from Rust:
//   enterScope();      // bind EventLoop to this thread
//   // ... poll user future (which may .await KJ promises) ...
//   poll();            // drive queued KJ events (non-blocking)
//   leaveScope();      // unbind from thread
class KjRuntimeImpl {
 public:
  KjRuntimeImpl();
  ~KjRuntimeImpl();

  // Bind the EventLoop to the current thread by creating a WaitScope.
  // Must be paired with leaveScope(). Must not be called if already in scope.
  void enterScope();

  // Unbind the EventLoop from the current thread by destroying the WaitScope.
  // Must be called after enterScope(). Must not be called if not in scope.
  void leaveScope();

  // Drive the event loop non-blocking. Requires that enterScope() has been called.
  // Returns number of events processed.
  uint poll();

  // Check if the event loop has queued work.
  bool isRunnable();

 private:
  kj::EventLoop loop;
  kj::Maybe<kj::WaitScope> waitScope;
};

std::unique_ptr<KjRuntimeImpl> newKjRuntime();

}  // namespace kj_rs
