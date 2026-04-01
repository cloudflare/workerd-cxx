// TODO(now): Make this a library, drive test from Rust.
// TODO(now): Move as many cases as possible into kj-rs.

#include "kj-rs-demo/test-promises.h"
#include "kj-rs/awaiter.h"
#include "kj-rs/future.h"
#include "kj-rs/tests/lib.rs.h"
#include "kj-rs/waker.h"

#include <sys/types.h>

#include <kj/test.h>

namespace kj_rs_demo {
namespace {

KJ_TEST("polling pending future") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  kj::Promise<void> promise = new_pending_future_void();
  KJ_EXPECT(!promise.poll(waitScope));
}

KJ_TEST("C++ KJ coroutine can co_await rust ready void future") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_ready_future_void(); }().wait(waitScope);
}

KJ_TEST("C++ KJ coroutines can co_await Rust Futures") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> {
    co_await new_ready_future_void();
    co_await new_waking_future_void(CloningAction::None, WakingAction::WakeByRefSameThread);
  }().wait(waitScope);
}

KJ_TEST("c++ can receive synchronous wakes during poll()") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  struct Actions {
    CloningAction cloningAction;
    WakingAction wakingAction;
  };

  for (auto testCase: std::initializer_list<Actions>{
         {CloningAction::None, WakingAction::WakeByRefSameThread},
         {CloningAction::None, WakingAction::WakeByRefBackgroundThread},
         {CloningAction::CloneSameThread, WakingAction::WakeByRefSameThread},
         {CloningAction::CloneSameThread, WakingAction::WakeByRefBackgroundThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeByRefSameThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeByRefBackgroundThread},
         {CloningAction::CloneSameThread, WakingAction::WakeSameThread},
         {CloningAction::CloneSameThread, WakingAction::WakeBackgroundThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeSameThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeBackgroundThread},
         {CloningAction::WakeByRefThenCloneSameThread, WakingAction::WakeSameThread},
       }) {
    auto waking = new_waking_future_void(testCase.cloningAction, testCase.wakingAction);
    KJ_EXPECT(waking.poll(waitScope));
    waking.wait(waitScope);
  }
}

KJ_TEST("RustPromiseAwaiter: Rust can .await KJ promises under a co_await") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_layered_ready_future_void(); }().wait(waitScope);
}

KJ_TEST("RustPromiseAwaiter: Rust can poll() multiple promises under a single "
        "co_await") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_naive_select_future_void(); }().wait(waitScope);
}

// TODO(now): Similar to "Rust can poll() multiple promises ...", but poll() until all are ready.

// TODO(now): Test polling a Promise from Rust with multiple LazyArcWakers.
//   Need a function which:
//   - Creates an OwnPromiseNode which is fulfilled manually.
//   - Wraps OwnPromiseNode::into_future() in BoxFuture.
//   - Passes the BoxFuture to a new KJ coroutine.
//   - The KJ coroutine passes the BoxFuture to a Rust function returning NaughtyFuture.
//   - The coroutine co_awaits the NaughtyFuture.
//   - The NaughtyFuture polls the BoxFuture once and returns Ready(BoxFuture).
//   - The coroutine co_returns the BoxFuture to the local function here.
//   - The BoxFuture has now outlived the coroutine which polled it last.
//   - Fulfill the OwnPromiseNode. Should not segfault.
//   - Pass the OwnPromiseNode to a new Rust Future somehow, .await it.

KJ_TEST("RustPromiseAwaiter: Rust can poll() KJ promises with non-KJ Wakers") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_wrapped_waker_future_void(); }().wait(waitScope);
}

KJ_TEST("co_awaiting a fallible future from C++ can throw") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> {
    kj::Maybe<kj::Exception> maybeException;
    try {
      co_await new_errored_future_void();
    } catch (...) {
      maybeException = kj::getCaughtExceptionAsKj();
    }
    auto& exception = KJ_ASSERT_NONNULL(maybeException, "should have thrown");
    KJ_EXPECT(exception.getDescription() == "test error");
  }().wait(waitScope);
}

KJ_TEST("co_awaiting a KjError future from C++ can throw with proper exception type") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> {
    kj::Maybe<kj::Exception> maybeException;
    try {
      co_await new_kj_errored_future_void();
    } catch (...) {
      maybeException = kj::getCaughtExceptionAsKj();
    }
    auto& exception = KJ_ASSERT_NONNULL(maybeException, "should have thrown");
    KJ_EXPECT(exception.getDescription() == "test error");
    KJ_EXPECT(exception.getType() == kj::Exception::Type::OVERLOADED);
  }().wait(waitScope);
}

KJ_TEST(".awaiting a Promise<T> from Rust can produce an Err Result") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_error_handling_future_void_infallible(); }().wait(
           waitScope);
}

KJ_TEST("Rust can await Promise<int32_t>") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_promise_i32_awaiting_future_void(); }().wait(waitScope);
}

KJ_TEST("C++ can await BoxFuture<i32>") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { KJ_EXPECT(co_await new_ready_future_i32(123) == 123); }().wait(
           waitScope);
}

KJ_TEST("C++ can receive asynchronous wakes after poll()") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto promise = new_threaded_delay_future_void();
  // It's not ready yet.
  KJ_EXPECT(!promise.poll(waitScope));
  // But later it is.
  promise.wait(waitScope);
}

KJ_TEST("Work before poll") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  uint64_t val = 0;
  // It should be possible for rust function to do work before returning the future
  // even if we don't poll or cancel it.
  auto promise = work_before_poll(val);
  KJ_EXPECT(val == 42);
}

// TODO(now): More test cases.
//   - Standalone ArcWaker tests. Ensure Rust calls ArcWaker destructor when we expect.
//   - Throwing an exception from PromiseNode functions, including destructor.

// =======================================================================================
// Cancellation tests
//
// In both KJ and Rust, dropping a promise/future synchronously cancels the underlying work. These
// tests verify that cancellation propagates correctly across the Rust/C++ async FFI boundary using
// a "cancellation-detecting promise" that increments a counter when destroyed.

KJ_TEST("Cancellation: drop never-polled Rust future") {
  // Dropping a kj::Promise wrapping a Rust future that was never polled should not crash. Since the
  // future was never polled, the Rust async function body was never entered, so no sub-promises
  // exist to cancel.
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  { auto promise = new_future_awaiting_cancellable_promise(); }
}

KJ_TEST("Cancellation: C++ dropping promise cancels Rust future's awaited KJ promise") {
  // When C++ drops a kj::Promise wrapping a Rust future that is currently .awaiting a KJ promise,
  // the cancellation should propagate through the Rust future to the inner KJ promise.
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  reset_cancellation_counter();

  {
    auto promise = new_future_awaiting_cancellable_promise();
    // Poll once to enter the Rust async function and suspend at the .await.
    KJ_EXPECT(!promise.poll(waitScope));
    KJ_EXPECT(get_cancellation_counter() == 0);
  }

  KJ_EXPECT(get_cancellation_counter() == 1);
}

KJ_TEST("Cancellation: propagates to current .await point in multi-step Rust future") {
  // A Rust future that has completed one .await and is suspended at a second should only cancel the
  // second sub-promise. The first has already completed and been consumed.
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  reset_cancellation_counter();

  {
    auto promise = new_two_step_cancellable_future();
    // Poll until the first step (coroutine promise) completes and the future suspends at the
    // second step (cancellation-detecting promise).
    KJ_EXPECT(!promise.poll(waitScope));
    KJ_EXPECT(get_cancellation_counter() == 0);
  }

  KJ_EXPECT(get_cancellation_counter() == 1);
}

KJ_TEST("Cancellation: Rust select cancels losing branch's KJ promise") {
  // When a Rust select() resolves one branch, the other branch is dropped, which should cancel its
  // KJ promise. This tests Rust-internal cancellation propagating to KJ promises.
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  reset_cancellation_counter();

  []() -> kj::Promise<void> { co_await new_select_with_cancellation(); }().wait(waitScope);

  // The coroutine promise won the select, so the cancellation-detecting promise was dropped.
  KJ_EXPECT(get_cancellation_counter() == 1);
}

KJ_TEST("Cancellation: Rust dropping never-polled KJ promise future") {
  // When Rust creates a PromiseFuture (by calling a C++ async fn) but drops it without ever
  // polling, the OwnPromiseNode is dropped directly by Rust, cancelling the KJ promise.
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  reset_cancellation_counter();

  []() -> kj::Promise<void> { co_await new_drop_cancellable_promise_without_polling(); }().wait(
           waitScope);

  KJ_EXPECT(get_cancellation_counter() == 1);
}

}  // namespace
}  // namespace kj_rs_demo