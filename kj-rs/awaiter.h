#pragma once

#include "kj-rs/waker.h"

#include <kj/debug.h>

namespace kj_rs {

// =======================================================================================
// Opaque Rust types
//
// The following types are defined in lib.rs, and thus in lib.rs.h. lib.rs.h depends on our C++
// headers, including awaiter.h (the file you're currently reading), so we forward-declare some types
// here for use in the C++ headers.

// Wrapper around an `&std::task::Waker`, passed to `RustPromiseAwaiter::poll()`. This indirection
// is required because cxx-rs does not permit us to expose opaque Rust types to C++ defined outside
// of our own crate, like `std::task::Waker`.
struct WakerRef;

// Wrapper around an `Option<std::task::Waker>`. RustPromiseAwaiter calls `set()` with the WakerRef
// passed to `poll()`. Later on, when its Promise becomes ready, RustPromiseAwaiter will use
// OptionWaker to call wake the wrapped Waker.
struct OptionWaker;

// =======================================================================================
// RustPromiseAwaiter

// RustPromiseAwaiter allows Rust `async` blocks to `.await` KJ promises. Rust code creates one in
// the block's storage at the point where the `.await` expression is evaluated, similar to how
// `kj::_::PromiseAwaiter` is created in the KJ coroutine frame when C++ `co_await`s a promise.
//
// To elaborate, RustPromiseAwaiter is part of the IntoFuture trait implementation for the
// OwnPromiseNode class, and `.await` expressions implicitly call `.into_future()`. So,
// RustPromiseAwaiter can be thought of a "Promise-to-Future" adapter. This also means that
// RustPromiseAwaiter can be constructed outside of `.await` expressions, and potentially _not_
// driven to complete readiness. Our implementation must be able to handle this case.
//
// Rust knows how big RustPromiseAwaiter is because we generate a Rust type of equal size and
// alignment using bindgen. See inside awaiter.c++ for a static_assert to remind us to re-run
// bindgen.
//
// RustPromiseAwaiter is a KJ Event. We use the Event to discover when our wrapped Promise is
// ready. Our Event fire() implementation records the fact that we are done, then wakes our Waker.
class RustPromiseAwaiter final: public kj::_::Event {
 public:
  // The Rust code which constructs RustPromiseAwaiter passes us a pointer to a OptionWaker, which can
  // be thought of as a Rust-native component RustPromiseAwaiter. Its job is to hold a clone of
  // of any non-KJ Waker that we are polled with, and forward calls to `wake()`. Ideally, we could
  // store the clone of the Waker ourselves (it's just two pointers) on the C++ side, so the
  // lifetime safety is more obvious. But, storing a reference works for now.
  RustPromiseAwaiter(
      OptionWaker& optionWaker, OwnPromiseNode node, kj::SourceLocation location = {});
  ~RustPromiseAwaiter() noexcept(false);
  KJ_DISALLOW_COPY_AND_MOVE(RustPromiseAwaiter);

  // -------------------------------------------------------
  // kj::_::Event API

  kj::Maybe<kj::Own<kj::_::Event>> fire() override;
  void traceEvent(kj::_::TraceBuilder& builder) override;

  // Helper for FuturePollEvent to report what promise it's waiting on.
  void tracePromise(kj::_::TraceBuilder& builder, bool stopAtNextEvent);

  // -------------------------------------------------------
  // API exposed to Rust code

  // Poll this Promise for readiness.
  bool poll(const WakerRef& waker);

  // Release ownership of the OwnPromiseNode. Asserts if called before the Promise is ready; that
  // is, `poll()` must have returned true prior to calling `take_own_promise_node()`.
  OwnPromiseNode take_own_promise_node();

 private:
  // The Rust code which instantiates RustPromiseAwaiter does so with a OptionWaker object right
  // next to the RustPromiseAwaiter, such that it is dropped after RustPromiseAwaiter. Thus, our
  // reference to our OptionWaker is stable. We use the OptionWaker to (optionally) store a clone of
  // the Waker with which we were last polled.
  //
  // When we wake our enclosing Future we nullify this Maybe. Therefore, this Maybe being kj::none
  // means our OwnPromiseNode is ready, and it is safe to call `node->get()` on it.
  kj::Maybe<OptionWaker&> maybeOptionWaker;

  kj::UnwindDetector unwindDetector;
  OwnPromiseNode node;
};

using PtrRustPromiseAwaiter = RustPromiseAwaiter*;

void rust_promise_awaiter_new_in_place(
    PtrRustPromiseAwaiter, OptionWaker*, OwnPromiseNode);
void rust_promise_awaiter_drop_in_place(PtrRustPromiseAwaiter);

// =======================================================================================
// FuturePollEvent

// Base class for `FutureAwaiter<F>`. `FutureAwaiter<F>` implements the type-specific
// `Event::fire()` override which actually polls the Future; this class implements all other base
// class virtual functions.
class FuturePollEvent: public kj::_::PromiseNode,
                       public kj::_::Event {
 public:
  FuturePollEvent(kj::SourceLocation location = {}): Event(location) {}

  // -------------------------------------------------------
  // PromiseNode API
  //
  // HACK: We only implement this interface for `tracePromise()`, which is the only function
  // CoroutineBase uses on its `promiseNodeForTrace` reference.

  void tracePromise(kj::_::TraceBuilder& builder, bool stopAtNextEvent) override;

 protected:
  // PollScope is a LazyArcWaker which is associated with a specific FuturePollEvent, allowing
  // optimized Promise `.await`s. Additionally, PollScope's destructor arranges to await any
  // ArcWaker promise which was lazily created.
  //
  // Used by FutureAwaiter<T>, our derived class.
  class PollScope;

 private:
  // Private API for PollScope.
  void enterPollScope() noexcept;
  void exitPollScope(kj::Maybe<kj::Promise<void>> maybeLazyArcWakerPromise);

  kj::Maybe<OwnPromiseNode> arcWakerPromise;
};

class FuturePollEvent::PollScope: public LazyArcWaker {
 public:
  // `futurePollEvent` is the FuturePollEvent responsible for calling `Future::poll()`, and must
  // outlive this PollScope.
  PollScope(FuturePollEvent& futurePollEvent);
  ~PollScope() noexcept(false);
  KJ_DISALLOW_COPY_AND_MOVE(PollScope);

 private:
  FuturePollEvent& futurePollEvent;
};

// =======================================================================================
// FutureAwaiter

template <typename F>
concept Future = requires(F f) {
  typename F::Output;
  {
    f.poll(kj::instance<const KjWaker&>(),
        kj::instance<typename ::kj::_::ExceptionOr<typename F::Output>&>())
  } -> std::same_as<void>;
};

// FutureAwaiter<T> is a Future poll() Event, and is the inner implementation of our co_await
// syntax. It wraps a Future and captures a reference to its enclosing KJ coroutine, arranging
// to continuously call `Future::poll()` on the KJ event loop until the Future produces a
// result, after which it arms the enclosing KJ coroutine's Event.
template <Future F>
class FutureAwaiter final: public FuturePollEvent {
 public:
  FutureAwaiter(F future, kj::SourceLocation location = {})
      : FuturePollEvent(location),
        future(kj::mv(future)) {}
  ~FutureAwaiter() noexcept(false) {}
  KJ_DISALLOW_COPY_AND_MOVE(FutureAwaiter);

  // -------------------------------------------------------
  // Event API

  void traceEvent(kj::_::TraceBuilder& builder) override {
    // Just defer to our enclosing Coroutine. It will immediately call our CoAwaitWaker's
    // `tracePromise()` implementation.
    onReadyEvent.traceEvent(builder);
  }

  void get(kj::_::ExceptionOrValue& output) noexcept override {
    output.as<typename F::Output>() = kj::mv(result);
  }

  void destroy() override {
    freePromise(this);
  }

  void onReady(kj::_::Event* event) noexcept override {
    onReadyEvent.init(event);
    poll();
  }

 private:
  kj::Maybe<kj::Own<kj::_::Event>> fire() override {
    poll();
    return kj::none;
  }

  // Poll the wrapped Future and arm the event if future is ready.
  void poll() {
    if (isDone()) return;

    // TODO(perf): Check if we already have an ArcWaker from a previous suspension and give it to
    //   LazyArcWaker for cloning if we have the last reference to it at this point. This could save
    //   memory allocations, but would depend on making XThreadFulfiller and XThreadPaf resettable
    //   to really benefit.

    {
      PollScope pollScope(*this);

      future.poll(pollScope, result);
      if (isDone()) {
        onReadyEvent.arm();
      }
    }
  }

  bool isDone() const {
    return result.value != kj::none || result.exception != kj::none;
  }

  typename F::ExceptionOrValue result;
  F future;
  OnReadyEvent onReadyEvent;
};

}  // namespace kj_rs
