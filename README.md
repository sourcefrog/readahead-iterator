# Readahead adaptor for iterators

API Docs: https://docs.rs/readahead-iterator/

Examples: https://github.com/sourcefrog/readahead-iterator/tree/main/examples

Items are generated from the iterator in a separate thread, and returned to the
caller as a regular iterator, in the same order.

This is useful when the wrapped iterator does significant work that can be
parallelized with other work on the calling thread. For example, if both the
iterator and its client are CPU-intensive, they utilize separate cores. Or if
the iterator does blocking IO on multiple files, opening of later files can be
overlapped with processing of earlier files.

# Comparison with Rayon

[Rayon][rayon] offers much more powerful ways to parallelize iterators, but requires that
the amount of total work be known in advance. For example, this won't compile in Rayon:

```rust
use rayon::prelude::*;

fn main() {
    (0..)
        .into_par_iter()
        .take(100)
        .for_each(|x| println!("{}", x));
}
```

```
error[E0599]: the method `into_par_iter` exists for struct `std::ops::RangeFrom<{integer}>`, but its trait bounds were not satisfied
   --> src/main.rs:7:10
    |
  6 | /     (0..)
  7 | |         .into_par_iter()
    | |_________-^^^^^^^^^^^^^
    |
   ::: /home/mbp/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ops/range.rs:189:1
    |
189 |   pub struct RangeFrom<Idx> {
    |   ------------------------- doesn't satisfy `_: IntoParallelIterator` or `_: ParallelIterator`
    |
    = note: the following trait bounds were not satisfied:
            `std::ops::RangeFrom<{integer}>: rayon::iter::ParallelIterator`
            which is required by `std::ops::RangeFrom<{integer}>: rayon::iter::IntoParallelIterator`
            `&std::ops::RangeFrom<{integer}>: rayon::iter::ParallelIterator`
            which is required by `&std::ops::RangeFrom<{integer}>: rayon::iter::IntoParallelIterator`
            `&mut std::ops::RangeFrom<{integer}>: rayon::iter::ParallelIterator`
            which is required by `&mut std::ops::RangeFrom<{integer}>: rayon::iter::IntoParallelIterator`
```

The equivalent code *does* work with `readahead-iterator`:

```rust
// examples/unbounded-input.rs
use readahead_iterator::IntoReadahead;

fn main() {
    (0..)
        .into_iter()
        .readahead(3)
        .take(100)
        .for_each(|x| println!("{}", x));
}
```

readahead-iterator also has some differences, which may or may not be desirable for your situation:

- Rayon processes items in arbitrary order; this crate processes them sequentially.
- Rayon distributes work across NCPUS threads and dynamically balances work across them. This crates currently uses only a single additional thread (but this could change.)
- Rayon is intended for CPU-bound work. This crate is also useful for IO-bound work that you want to hand off to a thread.

Both Rayon and threaded-readahead have few dependencies and enforce thread safety.
  
[rayon]: https://docs.rs/rayon/
