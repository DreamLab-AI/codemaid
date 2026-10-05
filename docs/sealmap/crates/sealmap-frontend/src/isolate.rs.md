---
sealmap: 2
source: crates/sealmap-frontend/src/isolate.rs
module: "sym:cargo sealmap_frontend . isolate/"
language: rust
source_hash: blake3:a12cb5bc9f2837cae6ee04dfa96a86529d06ec3f9227124b800bd72eb8ac98fe
lines: 75
fragments: 2
---
# `sym:cargo sealmap_frontend . isolate/` · crates/sealmap-frontend/src/isolate.rs
> Per-file collection on big stacks with a panic guard.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_frontend__isolate["sealmap_frontend::isolate"] {
    <<module>>
    +const COLLECT_STACK_BYTES: usize
    +map_isolated(items: &[T], stack_bytes: usize, job: F, on_panic: P) Vec#lt;R#gt;
    +panic_message(payload: &#40;dyn Any + Send#41;) String
  }
```

## `sym:cargo sealmap_frontend . isolate/map_isolated().`
`pub fn map_isolated<T, R, F, P>(items: &[T], stack_bytes: usize, job: F, on_panic: P) -> Vec<R> where T: Sync, R: Send, F: Fn(&T) -> R + Sync, P: Fn(&T, String) -> R + Sync,` · L31-L65
> Map `job` over `items`, in input order, on threads with `stack_bytes` of stack.
```mermaid
sequenceDiagram
  participant sealmap_frontend__isolate as isolate mod
  participant _rayon as rayon ext
  opt closure
    opt via unwrap_or_else
      sealmap_frontend__isolate->>sealmap_frontend__isolate: panic_message(as_ref())
    end
  end
  sealmap_frontend__isolate->>_rayon: ThreadPoolBuilder::ThreadPoolBuilder::new()
  opt let Ok(pool) = rayon::ThreadPoolBuilder::new().stack…
    sealmap_frontend__isolate->>_rayon: ThreadPoolBuilder::install(|..|)
    Note over sealmap_frontend__isolate: return pool.install(| | items.par_iter().map(one).colle…
  end
```
