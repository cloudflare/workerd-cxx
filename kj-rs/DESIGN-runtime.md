# Design: KJ Runtime as a Rust Future

## Problem

All async tests in kj-rs must be driven from C++ using the `KJ_TEST` macro. Rust has
no way to create or drive a `kj::EventLoop`, so it cannot independently run async tests
that `.await` KJ promises. The test at `kj-rs/tests/lib.rs:364` comments: *"these
promises can't be driven by rust side"*.

More broadly, there is no way to embed a KJ event loop inside a Rust async runtime
(e.g. Tokio). This limits the ability to write Rust-first services that interoperate
with KJ-based C++ libraries.

## Solution

Wrap `kj::EventLoop` (+ optionally `kj::EventPort`) as a Rust type (`KjRuntime`) with
an `async fn run()` method that interleaves polling a user-provided Rust `Future` with
driving the KJ event loop via `kj::WaitScope::poll()`.

On Linux, integrate with Tokio's reactor via `AsyncFd` watching the
`UnixEventPort::getPollableFd()` epoll file descriptor, enabling KJ I/O (timers,
sockets) to be driven from Rust.

## Architecture

```
+-- Tokio Runtime (or any async runtime) ----------------------+
|                                                               |
|  +-- KjRuntime::run() Future ------------------------------+ |
|  |                                                          | |
|  |   +-- User's async block (e.g. test body) -----------+  | |
|  |   |  ffi::new_coroutine_promise_void().await         |  | |
|  |   |    '-- PromiseFuture / RustPromiseAwaiter        |  | |
|  |   |         '-- registers Event on KJ EventLoop      |  | |
|  |   +--------------------------------------------------+  | |
|  |                                                          | |
|  |   KJ EventLoop  (driven by WaitScope::poll())           | |
|  |   +-- Event Queue ------------------------------------+ | |
|  |   |  coroutine steps, RustPromiseAwaiter firings,     | | |
|  |   |  cross-thread Executor events                     | | |
|  |   +--------------------------------------------------+ | |
|  |                                                          | |
|  |   [Linux] AsyncFd watching EventPort's epoll FD          | |
|  +----------------------------------------------------------+ |
+---------------------------------------------------------------+
```

## Key Design Decisions

### UniquePtr + unsafe impl Send

The original plan was to use `KjOwn<T>` (which is `Send` when `T: Send`). However,
CXX's proc macro resolves `KjOwn` to `::kj_rs::repr::KjOwn` — an absolute crate path
that doesn't work from within the `kj_rs` crate itself. The bridge declarations were
moved into `lib.rs`'s existing `#[cxx::bridge]` block and use `UniquePtr<KjRuntimeImpl>`
instead. `Send` is provided via:

```rust
unsafe impl Send for ffi::KjRuntimeImpl {}
unsafe impl Send for KjRuntime {}
```

**TODO**: Fix the CXX proc macro to resolve `KjOwn` via `crate::repr::KjOwn` (or
similar) when expanding inside the `kj_rs` crate, then switch back to
`KjOwn<KjRuntimeImpl>`. This would eliminate the manual `unsafe impl Send` on the
wrapper, since `KjOwn<T>` is `Send` when `T: Send`. The fix likely needs to be in
`syntax/tokens.rs:79` where the `::kj_rs::repr::` prefix is hardcoded.

### WaitScope lifecycle: enterScope/leaveScope

`kj::WaitScope` manages the thread-local binding between an `EventLoop` and the current
thread (`async.c++:1874-1884`):
- Constructor calls `enterScope()` which sets `threadLocalEventLoop = this`
- Destructor calls `leaveScope()` which sets `threadLocalEventLoop = nullptr`

The `EventLoop` constructor itself does NOT touch thread-local storage
(`async.c++:1761`). An `EventLoop` can exist on the heap, unbound to any thread,
indefinitely.

The original plan was to create/destroy a `WaitScope` entirely within the C++ `poll()`
call. This doesn't work: the user's Rust future may `.await` KJ promises during its
`poll()`, which requires an active `WaitScope` (the thread-local event loop pointer must
be set). The `WaitScope` must be alive during **both** the user future poll and the KJ
event loop drive.

The solution is `enterScope()`/`leaveScope()` methods that construct/destroy a
`kj::Maybe<kj::WaitScope>` member. Rust calls `enterScope()` at the start of each
`poll_with()`, polls the user future and drives KJ events while the scope is active,
then calls `leaveScope()` before returning. A `catch_unwind` guard ensures `leaveScope()`
is called even on panic.

Between `poll_with()` calls, the `WaitScope` does not exist and the `EventLoop` is
unbound. This enables:
- Transfer of the `EventLoop` between threads (KJ EventLoops are designed for this)
- Compatibility with Tokio's work-stealing scheduler (`tokio::spawn()`)

### Send safety

KJ EventLoops are transferrable between threads in production. In C++, thread-safety is
honored by convention: code running on an EventLoop avoids thread-locals and instead uses
"EventLoop-locals". The existing kj-rs Future types are `!Send` in Rust's type system,
but are in practice run on different threads via KJ's EventLoop transfer mechanism. This
is an intentional pragmatic tradeoff.

For `KjRuntimeImpl`, we apply the same convention:
```rust
unsafe impl Send for ffi::KjRuntimeImpl {}
```

This is sound because:
- `EventLoop` is a plain data structure (event queue, daemons)
- Thread-local binding is managed entirely by `WaitScope`, which is stack-scoped
- Cross-thread wakeups (`ArcWaker`, `CrossThreadPromiseFulfiller`) use atomic/mutex
  synchronization internally
- Between `poll()` calls, `threadLocalEventLoop == nullptr` and no Events are being
  armed or disarmed

## Phase 1: Non-I/O KJ Runtime (Cross-Platform)

**Goal**: Enable `#[tokio::test]` async tests that can `.await` KJ promises from Rust.

### Public API

```rust
// kj-rs/runtime.rs

/// A KJ event loop that can be driven as a Rust Future.
pub struct KjRuntime {
    inner: UniquePtr<ffi::KjRuntimeImpl>,
}

unsafe impl Send for KjRuntime {}

impl KjRuntime {
    /// Create a new KJ event loop (no I/O support).
    pub fn new() -> Self;

    /// Run a future with this KJ event loop active on the current thread.
    /// The future can .await KJ promises.
    pub async fn run<F: Future>(&mut self, f: F) -> F::Output;
}
```

### The poll() Algorithm

```
KjRuntime::poll_with(self, user_future, cx):
    enterScope()               // bind EventLoop to this thread

    loop:
        1. Poll user future with cx.waker()
           -> If Ready: leaveScope(); return Ready(value)
           -> If Pending: continue

        2. Drive KJ event loop: runtime.poll()
              (calls ws.poll() on the active WaitScope)
           -> processes all queued Events (coroutine steps, promise
              resolutions), may call our Waker synchronously when a
              RustPromiseAwaiter fires

        3. if runtime.is_runnable():
              continue       // more Events queued, re-poll user future

        4. leaveScope()        // unbind before returning Pending
           return Pending
           // the user future stored cx.waker() in the RustPromiseAwaiter;
           // when a KJ promise resolves or a background thread calls
           // waker.wake(), the outer runtime re-polls us from step 1
```

### C++ KjRuntimeImpl

```cpp
// kj-rs/runtime.h
#pragma once
#include <kj/async.h>
#include <kj/common.h>
#include <kj/memory.h>
#include <memory>

namespace kj_rs {

class KjRuntimeImpl {
public:
    KjRuntimeImpl();
    ~KjRuntimeImpl();

    void enterScope();           // bind EventLoop to current thread
    void leaveScope();           // unbind from current thread
    uint poll();                 // drive events (requires active scope)
    bool isRunnable();           // check for queued work

private:
    kj::EventLoop loop;
    kj::Maybe<kj::WaitScope> waitScope;
};

std::unique_ptr<KjRuntimeImpl> newKjRuntime();

} // namespace kj_rs
```

### Test Example

```rust
#[tokio::test]
async fn rust_can_await_kj_promises() {
    let mut kj = KjRuntime::new();
    kj.run(async {
        ffi::new_ready_promise_void().await.unwrap();
        ffi::new_coroutine_promise_void().await.unwrap();
    }).await;
}
```

### New Files

| File | Purpose |
|------|---------|
| `kj-rs/runtime.h` | C++ `KjRuntimeImpl` class (EventLoop wrapper) |
| `kj-rs/runtime.c++` | C++ implementation |
| `kj-rs/runtime.rs` | Rust module: CXX bridge, `KjRuntime` struct, `run()` impl |
| `kj-rs/tests/test_awaitables.rs` | Rust `#[tokio::test]` async tests |

### Modified Files

| File | Change |
|------|--------|
| `kj-rs/lib.rs` | Add `pub mod runtime;`, add CXX bridge declarations for `KjRuntimeImpl` |
| `kj-rs/BUILD.bazel` | Update deps (bridge declarations in `lib.rs`, no separate bridge rule) |
| `kj-rs/tests/lib.rs` | Add `mod test_awaitables;` |
| `kj-rs/tests/BUILD.bazel` | Add tokio dep to `tests` rust_library |
| `third-party/cargo.bzl` | Add tokio to PACKAGES |
| `third-party/Cargo.toml` | Add tokio (for IDE support) |
| `third-party/bazel/*` | Auto-regenerated by `bazel run //third-party:vendor` |

### Tests to Port from awaitables-cc-test.c++

| C++ Test | Rust Equivalent |
|----------|-----------------|
| "C++ KJ coroutine can co_await rust ready void future" | Ready future basic round-trip |
| "c++ can receive synchronous wakes during poll()" | WakingFuture (11 cloning/waking combos) |
| "RustPromiseAwaiter: Rust can .await KJ promises under a co_await" | .await ready + coroutine KJ promises |
| "RustPromiseAwaiter: Rust can poll() multiple promises" | naive_select over KJ promises |
| "RustPromiseAwaiter: non-KJ Wakers" | WrappedWaker test |
| "co_awaiting a fallible future from C++ can throw" | Error propagation (Rust Err -> C++) |
| ".awaiting a Promise\<T\> from Rust can produce an Err Result" | Error propagation (C++ throw -> Rust Err) |
| "Rust can await Promise\<int32_t\>" | Typed promise .await |
| "C++ can await BoxFuture\<i32\>" | Typed future |
| "C++ can receive asynchronous wakes after poll()" | ThreadedDelayFuture cross-thread wake |

## Phase 2: AsyncFd + UnixEventPort (Linux)

**Goal**: Support KJ I/O (timers, sockets, file descriptors) from Rust on Linux.

### C++ Changes

Add `UnixEventPort` on Linux:

```cpp
class KjRuntimeImpl {
#if __linux__
    kj::UnixEventPort port;
    kj::EventLoop loop{port};
#else
    kj::EventLoop loop;
#endif

public:
    uint poll() {
        kj::WaitScope ws(loop);
        return ws.poll();
    }

    bool isRunnable() { return loop.isRunnable(); }

#if __linux__
    int getPollableFd() { return port.getPollableFd(); }
    void preparePollableFdForSleep() { port.preparePollableFdForSleep(); }
#endif
};
```

### Rust Changes

Wrap the epoll FD in Tokio's `AsyncFd`:

```rust
pub struct KjRuntime {
    inner: KjOwn<ffi::KjRuntimeImpl>,
    #[cfg(target_os = "linux")]
    async_fd: Option<AsyncFd<PollableFdWrapper>>,
}
```

The poll() algorithm gains a step 4b before returning Pending:

```
4b. [Linux only]
    runtime.prepare_pollable_fd_for_sleep()
    match async_fd.poll_read_ready(cx):
        Ready(guard) -> guard.clear_ready(); continue   // I/O arrived
        Pending      -> return Pending                   // tokio reactor will wake us
```

On non-Linux, falls back to Phase 1 behavior (Waker-only notification).

### How Nested Epoll Works

Tokio's reactor uses epoll internally (via mio). `AsyncFd` registers the KJ
`UnixEventPort`'s epoll FD inside Tokio's epoll — nesting depth 2, well within
Linux's limit of 5.

Key constraint: mio uses `EPOLLET` (edge-triggered). The `WaitScope::poll()` call
fully drains KJ's event queue, satisfying the edge-triggered requirement that we
consume all available events before re-arming.

`preparePollableFdForSleep()` sets up timerfd and calls `wake()` if the loop has
queued work, ensuring the epoll FD becomes readable when there's something to do.

### Constraints

- `preparePollableFdForSleep()` throws if `onSignal()` waiters are active
- Tokio becomes a library dependency (for `AsyncFd`); recommend a separate Bazel
  target `//kj-rs:kj-rs-tokio` to avoid pulling it into all consumers

## Phase 2a: macOS I/O (kqueue)

**Goal**: Extend I/O integration to macOS using the same `AsyncFd` approach as Linux.

KJ's `UnixEventPort` already uses kqueue on macOS (`KJ_USE_KQUEUE`), and kqueue FDs
are pollable objects — just like epoll FDs on Linux. However, `getPollableFd()` is
currently only implemented for the epoll backend. There is a TODO in the KJ source
(`async-unix.h:202`):

> *"TODO(someday): Currently this is only implemented for epoll, NOT for kqueue. But
> in principle it should be possible to implement for kqueue as well."*

### Changes Required

1. **KJ upstream (capnp-cpp)**: Add `getPollableFd()` and
   `preparePollableFdForSleep()` to the kqueue backend of `UnixEventPort`. The
   kqueue fd is already stored as `kqueueFd`; `getPollableFd()` would return it.
   `preparePollableFdForSleep()` needs a kqueue-equivalent of the timerfd arming
   logic (likely using `EVFILT_TIMER`).

2. **kj-rs**: Change the `#[cfg(target_os = "linux")]` guards in Phase 2 to
   `#[cfg(unix)]` (or `#[cfg(any(target_os = "linux", target_os = "macos"))]`).
   `AsyncFd` is available on all Unix platforms, not just Linux.

3. **C++ side**: Change `#if __linux__` guards to `#if KJ_USE_EPOLL || KJ_USE_KQUEUE`
   (or simply `#if !_WIN32`).

### Complexity

Low-to-moderate. The architecture is identical to Linux; only the KJ upstream change
is nontrivial.

## Phase 2b: Windows I/O (Dedicated Wake Thread)

**Goal**: Support KJ I/O on Windows without deep reactor integration.

KJ has full Windows IOCP support via `Win32IocpEventPort` (`async-win32.h`). However,
there is no equivalent of `getPollableFd()` on Windows — IOCP handles are not pollable
objects that can be nested inside another reactor. Tokio's `AsyncFd` is Unix-only
(requires `AsRawFd`).

### Approach: Dedicated Thread + Cross-Thread Wake

Spawn a background thread that blocks on `Win32IocpEventPort::wait()`. When it returns
(meaning KJ I/O events have arrived), it wakes the Tokio task via a
`tokio::sync::Notify` (or the Waker directly).

```
+-- Tokio Runtime ------------------------------------------+
|                                                            |
|  KjRuntime::poll_with():                                   |
|    enterScope()                                            |
|    poll user future                                        |
|    WaitScope::poll()      // drive KJ events               |
|    leaveScope()                                            |
|    ... return Pending ...                                  |
|                                                            |
|  +-- Background Thread (Win32 only) ---+                   |
|  |  loop:                              |                   |
|  |    Win32IocpEventPort::wait()       |  --wake-->  Waker |
|  +-------------------------------------+                   |
+------------------------------------------------------------+
```

### Changes Required

1. **C++ side**: On Windows, `KjRuntimeImpl` holds a `Win32IocpEventPort` member
   (gated by `#if _WIN32`). Expose a blocking `waitForIo()` method for the background
   thread to call.

2. **Rust side**: On Windows, `KjRuntime::new()` spawns a `std::thread` that calls
   `waitForIo()` in a loop, waking the Tokio task each time. The thread is joined on
   drop.

3. **Synchronization**: The background thread must not call `waitForIo()` while
   `enterScope()` is active on another thread. A mutex or flag coordinates this.

### Tradeoffs

- Adds one OS thread per `KjRuntime` instance (cheap — it's blocked in a syscall).
- Event-driven latency (no polling interval).
- No forking of Tokio or mio required.
- KJ owns its own IOCP, Tokio owns its own IOCP — they don't interfere.

## Long-Term Alternative: Tokio-Backed EventPort

A more ambitious approach (not currently planned) would be to implement a custom
`kj::EventPort` that delegates to Tokio's reactor instead of owning platform-specific
I/O primitives (epoll, kqueue, IOCP).

KJ sockets and handles would be registered with Tokio's reactor (via `tokio::net` or
mio). The `EventPort::wait()` implementation would block on a channel or condvar
signaled by Tokio when KJ-relevant I/O completes. KJ would never create its own epoll
fd, kqueue fd, or IOCP handle.

**Advantages**: Cross-platform by construction — a single `EventPort` implementation
works on Linux, macOS, and Windows, since Tokio/mio already abstracts over
platform-specific I/O. Eliminates the background thread on Windows and the need for
`getPollableFd()` on any platform.

**Disadvantages**: Tight coupling between KJ and Tokio. Significant implementation
effort — all KJ I/O operations would need to flow through Tokio's abstractions.
Behavioral differences between KJ's and Tokio's I/O models (edge-triggered vs.
level-triggered, signal handling, etc.) could surface as subtle bugs.

**Does not require forking Tokio or mio** — this would be a new `EventPort`
implementation in kj-rs code, not modifications to upstream projects.

## Multi-Task Spawning (Future Work)

The `run()` API drives a single future. For dynamic task spawning (e.g. a connection
accept loop), there are several approaches to consider in the future:

### A. Internal concurrency with Rust combinators

```rust
kj.run(async {
    let mut tasks = FuturesUnordered::new();
    loop {
        tokio::select! {
            conn = accept() => {
                tasks.push(handle_connection(conn));
            }
            Some(result) = tasks.next() => { /* task done */ }
        }
    }
}).await;
```

Works today with no API changes.

### B. KJ-native TaskSet via FFI

```rust
kj.run(async {
    let task_set = ffi::new_task_set();
    loop {
        let conn = accept().await;
        task_set.add(handle_connection(conn));
    }
}).await;
```

Natural for KJ-centric workloads. Requires bridging `kj::TaskSet`.

### C. KjRuntimeHandle + spawn()

```rust
let handle = kj.handle();
kj.run(async move {
    loop {
        let conn = accept().await;
        handle.spawn(async move {
            handle_connection(conn).await;
        });
    }
}).await; // completes when main future + all spawned tasks finish
```

Most ergonomic for Rust. Modeled after Tokio's `LocalSet`.

## Implementation Order

1. Add tokio to `third-party/cargo.bzl` + regenerate vendor
2. Create `kj-rs/runtime.h` + `kj-rs/runtime.c++`
3. Create `kj-rs/runtime.rs` (CXX bridge + KjRuntime + run())
4. Wire up `kj-rs/BUILD.bazel`
5. Export from `kj-rs/lib.rs`
6. Create `kj-rs/tests/test_awaitables.rs` with `#[tokio::test]` tests
7. Update `kj-rs/tests/BUILD.bazel` + `kj-rs/tests/lib.rs`
8. Verify: `just test` passes (all existing tests + new Rust async tests)
