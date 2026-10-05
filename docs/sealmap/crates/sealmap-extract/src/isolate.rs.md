---
sealmap: 2
source: crates/sealmap-extract/src/isolate.rs
module: "sym:cargo sealmap_extract . isolate/"
language: rust
source_hash: blake3:ed9fd1c98c087578f93f13f26c3cddc9fcfed1e8234bf2483dcf2d4da4dbe610
lines: 75
fragments: 2
---
# `sym:cargo sealmap_extract . isolate/` · crates/sealmap-extract/src/isolate.rs
> Per-file collection on big stacks with a panic guard.

## structure
```mermaid
classDiagram
  direction LR
  class sealmap_extract__isolate["sealmap_extract::isolate"] {
    <<module>>
    +const COLLECT_STACK_BYTES: usize
    +map_isolated(items: &[T], stack_bytes: usize, job: F, on_panic: P) Vec#lt;R#gt;
    +panic_message(payload: &#40;dyn Any + Send#41;) String
  }
```

## `sym:cargo sealmap_extract . isolate/map_isolated().`
`pub fn map_isolated<T, R, F, P>(items: &[T], stack_bytes: usize, job: F, on_panic: P) -> Vec<R> where T: Sync, R: Send, F: Fn(&T) -> R + Sync, P: Fn(&T, String) -> R + Sync,` · L31-L65
> Map `job` over `items`, in input order, on threads with `stack_bytes` of stack.
```mermaid
sequenceDiagram
  participant sealmap_extract__isolate as isolate mod
  participant _rayon as rayon ext
  opt closure
    opt via unwrap_or_else
      sealmap_extract__isolate->>sealmap_extract__isolate: panic_message(as_ref())
    end
  end
  sealmap_extract__isolate->>_rayon: ThreadPoolBuilder::new()
  opt let Ok(pool) = rayon::ThreadPoolBuilder::new().stack…
    sealmap_extract__isolate->>_rayon: ThreadPoolBuilder::install(|..|)
    Note over sealmap_extract__isolate: return pool.install(| | items.par_iter().map(one).colle…
  end
```
