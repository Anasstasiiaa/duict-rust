#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(unreachable_pub)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::result_large_err)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::similar_names)]

//! # Practical Work 8
//!
//! Manual implementation of `Future`, `Waker` mechanics, and structural pinning without
//! using external async runtimes like Tokio.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake};
use std::thread;
use std::time::{Duration, Instant};

// =====================================================================
// 1. MeasurableFuture Implementation
// =====================================================================

/// A wrapper future that measures the execution time of its inner future.
pub struct MeasurableFuture<Fut> {
    inner_future: Fut,
    started_at: Option<Instant>,
}

impl<Fut> MeasurableFuture<Fut> {
    /// Creates a new `MeasurableFuture` wrapper.
    pub fn new(inner_future: Fut) -> Self {
        Self {
            inner_future,
            started_at: None,
        }
    }
}

impl<Fut: Future> Future for MeasurableFuture<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We are performing structural pinning manually.
        // This is sound because:
        // 1. We do not move the `inner_future` out of `Self`.
        // 2. We do not implement `Drop` that would invalidate the pinned state.
        let this = unsafe { self.get_unchecked_mut() };

        // Initialize the timer on the first poll
        if this.started_at.is_none() {
            this.started_at = Some(Instant::now());
        }

        // Project the pinned mutable reference to the inner future
        let inner_pin = unsafe { Pin::new_unchecked(&mut this.inner_future) };

        match inner_pin.poll(cx) {
            Poll::Ready(output) => {
                let elapsed = this.started_at.unwrap().elapsed();
                println!("MeasurableFuture: Execution completed in {:?}", elapsed);
                Poll::Ready(output)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// =====================================================================
// 2. Delay Future Implementation (Non-blocking Timer)
// =====================================================================

/// A Future that resolves after a specified duration.
pub struct Delay {
    target: Instant,
    is_waker_spawned: bool,
}

impl Delay {
    /// Creates a new `Delay` future that will be ready after `millis` milliseconds.
    pub fn new(millis: u64) -> Self {
        Self {
            target: Instant::now() + Duration::from_millis(millis),
            is_waker_spawned: false,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = Instant::now();

        if now >= self.target {
            Poll::Ready(())
        } else {
            // If the waker thread hasn't been spawned yet, spawn it.
            // This acts as our mini-reactor, preventing the main thread from blocking.
            if !self.is_waker_spawned {
                let waker = cx.waker().clone();
                let target = self.target;

                thread::spawn(move || {
                    let current = Instant::now();
                    if current < target {
                        thread::sleep(target - current);
                    }
                    // Wake the executor once the sleep is done
                    waker.wake();
                });

                self.is_waker_spawned = true;
            }
            Poll::Pending
        }
    }
}

// =====================================================================
// 3. Minimal Custom Executor
// =====================================================================

/// A simple Waker that unparks the thread it holds.
struct ThreadWaker {
    thread: thread::Thread,
}

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.thread.unpark();
    }
}

/// A minimal block_on executor to drive a Future to completion on the current thread.
pub fn block_on<F: Future>(future: F) -> F::Output {
    // Pin the future to the stack using the standard library macro
    let mut future = std::pin::pin!(future);

    let waker = Arc::new(ThreadWaker {
        thread: thread::current(),
    })
    .into();

    let mut cx = Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            // If pending, park the thread. It will consume no CPU cycles until unparked.
            Poll::Pending => thread::park(),
        }
    }
}

// =====================================================================
// Entry Point
// =====================================================================

fn main() {
    println!("Starting custom async runtime...");

    // Compose our futures
    let delay_future = Delay::new(1500); // 1.5 seconds delay
    let measurable_task = MeasurableFuture::new(delay_future);

    // Run the composed future in our custom executor
    block_on(measurable_task);

    println!("Runtime finished execution.");
}
