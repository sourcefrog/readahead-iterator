// Copyright 2020, 2021, 2025 Martin Pool
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Readahead adaptor for iterators.
//!
//! Items are generated from the iterator in a separate thread, and returned
//! to the caller as a regular iterator, in the same order.
//!
//! This is useful when the wrapped iterator does significant work that
//! can be parallelized with other work on the calling thread.
//!
//! For example:
//!
//! 1. Both the iterator and its client are CPU-intensive: allowing the iterator to
//!    run ahead will let it do some work in parallel on a separate core.
//! 2. The iterator generating work does blocking or lengthy IO such as opening
//!    and reading many files: opening the files can proceed in parallel with
//!    processing already-open files.
//!
//! The wrapped iterator (and its items) must be `Send` so that they can be
//! sent between threads.
//!
//! The iterator must also have `'static` lifetime, so that it lives long
//! enough for the thread and wrapper. Often this can be accomplished by
//! making sure the inner iterator is by-value, rather than iterating
//! references through a collection: construct it with
//! [`into_iter()`](https://doc.rust-lang.org/std/iter/index.html#the-three-forms-of-iteration).
//!
//! For example, to overlap opening files with reading from them:
//!
//! ```
//! use std::fs::File;
//! use std::io::{BufRead, BufReader};
//! use readahead_iterator::IntoReadahead;
//!
//! let filenames = vec!["src/lib.rs", "examples/linecount.rs", "Cargo.toml"];
//! let total_lines: usize = filenames
//!     .into_iter()
//!     .filter_map(|filename| {
//!         File::open(filename.clone())
//!             .map_err(|err| eprintln!("failed to open {}: {:?}", filename, err))
//!             .map(|file| (filename, file))
//!             .ok()
//!     })
//!     .readahead(5)
//!     .map(|(filename, file)| {
//!         let line_count = BufReader::new(file).lines().count();
//!         println!("{:>8} {}", line_count, filename);
//!         line_count
//!     })
//!     .sum();
//! println!("{:>8} TOTAL", total_lines);
//! ```
//!
//! # Potential future features:
//!
//! 1. A threaded `map` across a bounded readahead from the iterator, processing them
//!    out of order within a sliding window.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::sync::mpsc::{sync_channel, Receiver};
use std::thread;

/// An iterator adaptor that evaluates the iterator on a separate thread,
/// and transports the items back to be consumed from the original thread.
pub struct Readahead<T: Send + 'static> {
    receiver: Option<Receiver<Option<T>>>,
}

impl<T> Readahead<T>
where
    T: Send + 'static,
{
    /// Apply a threaded readahead to an iterator.
    ///
    /// Items from the iterator are produced on a separate thread and passed
    /// back to the calling thread.
    ///
    /// `buffer_size` is the maximum number of items that can be buffered.
    ///
    /// ```
    /// use readahead_iterator::Readahead;
    /// let c = Readahead::new("Hello Ferris".chars(), 10)
    ///     .filter(|c| c.is_uppercase())
    ///     .count();
    /// # assert_eq!(c, 2);
    /// ```
    ///
    /// # Panics
    ///
    /// On failing to spawn a new thread.
    pub fn new<I>(inner: I, buffer_size: usize) -> Self
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        let (sender, receiver) = sync_channel(buffer_size);
        thread::Builder::new()
            .name("readahead_iterator".to_owned())
            .spawn(move || {
                for item in inner {
                    if sender.send(Some(item)).is_err() {
                        // Receiver has been dropped, stop sending
                        return;
                    }
                }
                // Receiver has been dropped, no need to send final None
                let _ = sender.send(None);
            })
            .expect("failed to spawn readahead_iterator thread"); // TODO: Optionally return an error instead.
        Readahead {
            receiver: Some(receiver),
        }
    }
}

impl<T> Iterator for Readahead<T>
where
    T: Send + 'static,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        // Iterator returns None when:
        // 1. receiver is already None, i.e. we already ended.
        // 2. sender sent an explicit None indicating the end, i.e. normal termination
        // 3. the sender hung up: this shouldn't normally happen but let's not panic.
        let r = self
            .receiver
            .as_ref()
            .and_then(|r| r.recv().ok())
            .unwrap_or_default();
        if r.is_none() {
            self.receiver = None
        }
        r
    }
}

/// Adds a `.readahead(buffer_size)` method to any iterator.
///
/// ```
/// use readahead_iterator::IntoReadahead;
///
/// let c = "Some input data".chars()
///     .readahead(10)
///     .filter(|c| c.is_alphabetic())
///     .count();
/// # assert_eq!(c, 13);
/// ```
pub trait IntoReadahead<T>
where
    T: Send + 'static,
{
    /// Apply a readahead adaptor to an iterator.
    ///
    /// `buffer_size` is the maximum number of buffered items.
    fn readahead(self, buffer_size: usize) -> Readahead<T>
    where
        Self: Send + 'static;
}

impl<I, T> IntoReadahead<T> for I
where
    T: Send + 'static,
    I: Iterator<Item = T>,
{
    fn readahead(self, buffer_size: usize) -> Readahead<T>
    where
        Self: Send + 'static,
    {
        Readahead::new(self, buffer_size)
    }
}

#[cfg(test)]
mod test {
    use std::iter::once;

    use super::*;

    /// Test that we don't panic if the receiver thread quits unexpectedly.
    ///
    /// This might not be possible to construct through the public interface
    /// but it's still good to avoid a potential panic.
    #[test]
    fn sender_exits_unexpectedly() {
        let (sender, receiver) = sync_channel(4);
        thread::Builder::new()
            .spawn(move || {
                sender.send(Some(1)).expect("send failed");
            })
            .expect("failed to spawn readahead_iterator thread"); // TODO: Optionally return an error instead.
        let mut r = Readahead {
            receiver: Some(receiver),
        };
        assert_eq!(r.next(), Some(1));
        // the sender quit without returning None but we shouldn't panic: just see that as the end
        assert_eq!(r.next(), None);
        assert_eq!(r.next(), None);
    }

    #[test]
    fn receiver_doesnt_panic_if_sender_panics() {
        // TODO: Possibly some callers might want to propagate panics??
        //
        // Note: this will display a panic warning on the test's stderr, but the
        // calling thread continues on and succeeds.
        let vals = vec![false, true];
        let iter = vals.into_iter().map(|v| if v { panic!() } else { 2 });
        let mut r = iter.readahead(1);
        assert_eq!(r.next(), Some(2));
        assert_eq!(r.next(), None);
        assert_eq!(r.next(), None);
    }
}
