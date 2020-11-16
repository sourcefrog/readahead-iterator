// Copyright 2020 Martin Pool
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::mpsc::{sync_channel, Receiver};
use std::thread;

///
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
