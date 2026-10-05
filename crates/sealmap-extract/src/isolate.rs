//! Per-file collection on big stacks with a panic guard.
//!
//! Recursive-descent parsers and body walkers use stack in proportion to
//! syntactic nesting, and generated files can nest far deeper than a 2 MiB
//! thread stack allows (measured on syn: one level of `UInt<UInt<..>>` costs
//! about 40 KiB in a debug build). [`map_isolated`] therefore runs every job
//! on threads with [`COLLECT_STACK_BYTES`] of stack, and turns a panic in one
//! job into a value instead of aborting the run. A stack overflow is an abort
//! and cannot be caught, which is why the big stack (and a language adapter's own
//! depth cap) are what actually matter.
//!
//! ```
//! use sealmap_extract::isolate::{COLLECT_STACK_BYTES, map_isolated};
//!
//! let files = ["ok", "boom", "fine"];
//! let out = map_isolated(
//!     &files,
//!     COLLECT_STACK_BYTES,
//!     |f| if *f == "boom" { panic!("bad input") } else { f.len() },
//!     |_, why| { assert_eq!(why, "bad input"); 0 },
//! );
//! assert_eq!(out, [2, 0, 4]);
//! ```

use std::any::Any;

/// Default stack for collector threads. Pages are only committed when
/// touched, so the cost is address space (64 MiB × threads), not memory.
pub const COLLECT_STACK_BYTES: usize = 64 * 1024 * 1024;

/// Map `job` over `items`, in input order, on threads with `stack_bytes` of
/// stack. A panicking job yields `on_panic(item, message)` instead.
///
/// With the `parallel` feature the jobs run on a dedicated rayon pool;
/// otherwise (or if the pool cannot be built) on one scoped thread with the
/// same stack, so behaviour does not depend on the feature. If even that
/// thread cannot be spawned, the jobs run on the calling thread.
pub fn map_isolated<T, R, F, P>(items: &[T], stack_bytes: usize, job: F, on_panic: P) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync,
    P: Fn(&T, String) -> R + Sync,
{
    let one = |item: &T| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| job(item)))
            .unwrap_or_else(|e| on_panic(item, panic_message(e.as_ref())))
    };
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        if let Ok(pool) = rayon::ThreadPoolBuilder::new().stack_size(stack_bytes).build() {
            return pool.install(|| items.par_iter().map(one).collect());
        }
    }
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("sealmap-collect".into())
            .stack_size(stack_bytes)
            .spawn_scoped(scope, || items.iter().map(one).collect())
            .ok()
            .and_then(|h| h.join().ok())
    })
    .unwrap_or_else(|| items.iter().map(one).collect())
}

/// The message of a caught panic payload (`&str` or `String`), or
/// `unknown panic`.
pub fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".into())
}
