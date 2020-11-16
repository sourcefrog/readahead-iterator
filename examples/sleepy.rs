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

//! Simple demonstration of threaded readahead: generate integers slowly in
//! an iterator, and consume them also with some delay from the main thread.
//!
//! As expected, doing all the work on the main thread takes about
//! `N * (PROCESS_DELAY + GENERATE_DELAY)`, whereas running them in
//! parallel improves this to `N * max(PROCESS_DELAY, GENERATE_DELAY)`.

use std::thread::sleep;
use std::time::{Duration, Instant};

use readahead_iterator::Readahead;

const PROCESS_DELAY: Duration = Duration::from_millis(200);
const GENERATE_DELAY: Duration = Duration::from_millis(500);
const N: usize = 20;

fn slow_iterator() -> impl Iterator<Item = usize> {
    (0..N).inspect(|i| {
        println!("generating {}...", i);
        sleep(GENERATE_DELAY);
    })
}

pub fn main() {
    let start = Instant::now();
    println!("without readahead:");
    for i in slow_iterator() {
        println!("processing {}...", i);
        sleep(PROCESS_DELAY);
    }
    println!("elapsed: {:?}", Instant::now() - start);

    let start = Instant::now();
    println!("with readahead:");
    for i in Readahead::new(slow_iterator(), 10) {
        println!("processing {}...", i);
        sleep(PROCESS_DELAY);
    }
    println!("elapsed: {:?}", Instant::now() - start);
}
