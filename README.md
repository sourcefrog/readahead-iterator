# Readahead adaptor for iterators

Items are generated from the iterator in a separate thread, and returned
to the caller as a regular iterator, in the same order.

This is useful when the wrapped iterator does significant work that can be
parallelized with other work on the calling thread. For example, if both the
iterator and its client are CPU-intensive, they utilize separate cores. Or if
the iterator does blocking IO on multiple files, opening of later files can be
overlapped with processing of earlier files.

[Rayon][rayon] offers *much* more powerful ways to parallelize iterators. This
adaptor is useful for some simpler and complementary cases:

 * Both the producer and consumer are serial.

 * Items should be produced and consumed one at a time and in order, but
   production and consumption can be overlapped.

 * The work is potentially IO-bound, so a separate thread can be dedicated to
   this iterator, rather than using an `NCPUS` thread pool.

[rayon]: https://docs.rs/rayon/
