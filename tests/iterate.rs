// Copyright 2020 Martin Pool
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::thread::sleep;
use std::time::Duration;

use readahead_iterator::Readahead;

/// A lot like examples/sleepy, but with minimal sleeps.
#[test]
fn iterate_integers() {
    const N: usize = 100;
    let s: usize = Readahead::new(
        (0..N)
            .map(|i| i * 3)
            .inspect(|_| sleep(Duration::from_millis(1))),
        50,
    )
    .sum();
    assert_eq!(s, 3 * (N * (N - 1)) / 2);
}
