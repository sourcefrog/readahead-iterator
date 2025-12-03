// Copyright 2020, 2021, 2025 Martin Pool
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::thread::sleep;
use std::time::Duration;

use readahead_iterator::{IntoReadahead, Readahead};

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

/// Continuing to read after the enclosed function ends gets more Nones.
#[test]
fn read_past_end() {
    let mut rah = Readahead::new(
        (0..10)
            .map(|i| i)
            .inspect(|_| sleep(Duration::from_millis(10))),
        50,
    );
    for i in 0..20 {
        let v = rah.next();
        if i < 10 {
            assert_eq!(v, Some(i));
        } else {
            assert_eq!(v, None);
        }
    }
}

/// Taking exactly the number of available items should not panic.
#[test]
fn take_exact_count() {
    let values: Vec<_> = Readahead::new((0..10).inspect(|_| sleep(Duration::from_millis(1))), 5)
        .take(10)
        .collect();
    assert_eq!(values, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

/// Taking fewer items than available should not panic.
#[test]
fn take_fewer_items() {
    let values: Vec<_> = Readahead::new((0..10).inspect(|_| sleep(Duration::from_millis(1))), 5)
        .take(5)
        .collect();
    assert_eq!(values, vec![0, 1, 2, 3, 4]);
}

/// Taking more items than available should work fine.
#[test]
fn take_more_items() {
    let values: Vec<_> = Readahead::new((0..5).inspect(|_| sleep(Duration::from_millis(1))), 3)
        .take(10)
        .collect();
    assert_eq!(values, vec![0, 1, 2, 3, 4]);
}

#[test]
fn unbounded_input() {
    (0..)
        .into_iter()
        .readahead(3)
        .take(100)
        .for_each(|x| println!("{}", x));
}
