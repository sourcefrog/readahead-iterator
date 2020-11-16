// Copyright 2020 Martin Pool
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
