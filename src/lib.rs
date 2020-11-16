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
