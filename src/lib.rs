// Copyright 2020 Martin Pool
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
//! can be parallelized with other work on the calling thread. For example,
//! if both the iterator and its client are CPU-intensive, they utilize separate
//! cores. Or if the iterator does blocking IO on multiple files, opening of
//! later files can be overlapped with processing of earlier files.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::sync::mpsc::{sync_channel, Receiver};
use std::thread;

/// An iterator adaptor that evaluates the iterator on a separate thread,
/// and transports the items back to be consumed from the original thread.
///
/// This is useful when the iterator does IO or uses the CPU in a way that
/// can be parallelized with the main thread, while still letting the
/// caller consume items in order and synchronously.
pub struct Readahead<T: Send + 'static> {
    receiver: Receiver<Option<T>>,
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
    pub fn new<I>(inner: I, buffer_size: usize) -> Self
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        // TODO: What if the iterator is dropped?
        let (sender, receiver) = sync_channel(buffer_size);
        thread::spawn(move || {
            for item in inner {
                sender
                    .send(Some(item))
                    .expect("send from inner iterator failed");
            }
            sender.send(None).expect("send of final None failed");
        });
        Readahead { receiver }
    }
}

impl<T> Iterator for Readahead<T>
where
    T: Send + 'static,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.receiver.recv().expect("recv of iterator value failed")
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
